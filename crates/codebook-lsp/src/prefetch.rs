//! Background dictionary prefetch.
//!
//! The spell-check path never downloads (`get_dictionary` is disk-only), so
//! this worker greedily fetches every dictionary the config can resolve to —
//! plus all per-language word lists — on its own thread, retrying transient
//! failures with exponential backoff. When a Hunspell pair that previously
//! couldn't load lands on disk, it emits an event so the LSP can re-check
//! open documents; refreshes and text word lists are picked up by the next
//! natural check instead.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use codebook::{Codebook, EnsureOutcome};
use log::{debug, info, warn};

/// How long the worker waits between passes when nothing is awaiting retry.
/// A pass over fresh entries is pure disk reads, so re-walking is cheap; it
/// also catches the biweekly revalidation window lapsing while the server is
/// long-lived.
const IDLE_REWALK: Duration = Duration::from_secs(6 * 3600);

const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(30);
const MAX_RETRY_DELAY: Duration = Duration::from_secs(3600);

// Both channels carry unit messages: an event means "a dictionary that
// previously couldn't load is now available" (the listener re-checks open
// documents); a command means "kick: re-enumerate ids and retry transient
// failures now" (config changed).

pub struct PrefetchHandle {
    cmd_tx: Sender<()>,
}

impl PrefetchHandle {
    /// Wake the worker: re-enumerate dictionary ids and retry transient
    /// failures immediately.
    pub fn kick(&self) {
        let _ = self.cmd_tx.send(());
    }
}

/// Start the prefetch worker on a dedicated thread. Blocking HTTP can't live
/// on the (current-thread) tokio runtime, and a process-lifetime loop
/// shouldn't pin a `spawn_blocking` pool slot. The thread runs for the life
/// of the process; dropping the handle makes it exit at its next wakeup
/// (immediately if idle, after the in-flight pass otherwise).
pub fn spawn(
    codebook: Arc<Codebook>,
    events: tokio::sync::mpsc::UnboundedSender<()>,
) -> PrefetchHandle {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("codebook-prefetch".into())
        .spawn(move || {
            let mut backoff = Backoff::default();
            loop {
                let new_pair = run_pass(codebook.as_ref(), &mut backoff, Instant::now());
                if new_pair && events.send(()).is_err() {
                    break;
                }
                let wait = backoff
                    .next_retry_delay(Instant::now())
                    .unwrap_or(IDLE_REWALK);
                match cmd_rx.recv_timeout(wait) {
                    Ok(()) => backoff.reset_transient(),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            debug!("Prefetch worker exiting");
        })
        .expect("failed to spawn prefetch worker thread");
    PrefetchHandle { cmd_tx }
}

/// One walk over the wanted ids: fetch whatever isn't backing off, record
/// failures. Ids are re-enumerated each call so config changes are picked
/// up. Returns true when a previously unloadable Hunspell pair landed — the
/// recheck policy (only new pairs warrant one) lives here, in the layer
/// that owns rechecking.
fn run_pass(codebook: &Codebook, backoff: &mut Backoff, now: Instant) -> bool {
    let mut new_pair_downloaded = false;
    for id in codebook.prefetch_dictionary_ids() {
        if !backoff.ready(&id, now) {
            continue;
        }
        match codebook.ensure_dictionary(&id) {
            Ok(EnsureOutcome::NewHunspellPair) => {
                info!("Dictionary '{id}' downloaded");
                backoff.clear(&id);
                new_pair_downloaded = true;
            }
            Ok(_) => backoff.clear(&id),
            Err(e) if e.permanent => {
                warn!("Dictionary '{id}' does not exist upstream, not retrying: {e}");
                backoff.mark_permanent(&id);
            }
            Err(e) => {
                // `now` is the pass start, so a pass that outlasts the
                // current delay retries immediately once — self-limiting,
                // since the delay doubles per failed pass.
                let delay = backoff.record_failure(&id, now);
                warn!(
                    "Failed to fetch dictionary '{id}', next attempt in {}s: {e}",
                    delay.as_secs()
                );
            }
        }
    }
    new_pair_downloaded
}

/// Per-dictionary retry state. Every method takes `now` instead of reading a
/// clock, so tests never sleep.
#[derive(Default)]
struct Backoff {
    /// id → (failed attempts so far, earliest next attempt)
    transient: HashMap<String, (u32, Instant)>,
    permanent: HashSet<String>,
}

impl Backoff {
    fn delay_for(attempts: u32) -> Duration {
        INITIAL_RETRY_DELAY
            .saturating_mul(1u32 << attempts.min(7))
            .min(MAX_RETRY_DELAY)
    }

    fn ready(&self, id: &str, now: Instant) -> bool {
        if self.permanent.contains(id) {
            return false;
        }
        match self.transient.get(id) {
            Some((_, next_at)) => *next_at <= now,
            None => true,
        }
    }

    fn clear(&mut self, id: &str) {
        self.transient.remove(id);
    }

    fn record_failure(&mut self, id: &str, now: Instant) -> Duration {
        let attempts = self.transient.get(id).map(|(a, _)| a + 1).unwrap_or(0);
        let delay = Self::delay_for(attempts);
        self.transient.insert(id.to_string(), (attempts, now + delay));
        delay
    }

    fn mark_permanent(&mut self, id: &str) {
        self.transient.remove(id);
        self.permanent.insert(id.to_string());
    }

    /// A config change is the user's "retry now": transient failures reset.
    /// Permanent 404s survive — a kick can't make an upstream dictionary
    /// exist. (An LSP restart clears those too.)
    fn reset_transient(&mut self) {
        self.transient.clear();
    }

    /// Time until the soonest transient retry, if any.
    fn next_retry_delay(&self, now: Instant) -> Option<Duration> {
        self.transient
            .values()
            .map(|(_, next_at)| next_at.saturating_duration_since(now))
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codebook::dictionaries::manager::DictionaryManager;
    use codebook_config::CodebookConfigMemory;
    use codebook_downloader::testing::{FakeTransport, ScriptedResult, connection_error, ok};
    use tempfile::TempDir;

    /// A Codebook over a scripted transport. With the default (memory)
    /// config, the prefetch set is en_us plus the downloadable text lists:
    /// 6 text requests + 2 for the en_us aff/dic pair = 8 for a full pass.
    fn codebook_with(script: Vec<ScriptedResult>) -> (Codebook, Arc<FakeTransport>, TempDir) {
        let transport = FakeTransport::new(script);
        let cache_dir = TempDir::new().unwrap();
        let manager = DictionaryManager::with_transport(
            &cache_dir.path().to_path_buf(),
            None,
            transport.clone(),
        );
        let config = Arc::new(CodebookConfigMemory::default());
        (
            Codebook::with_manager(config, manager),
            transport,
            cache_dir,
        )
    }

    fn all_ok(n: usize) -> Vec<ScriptedResult> {
        (0..n).map(|_| ok(200, "word\n", None)).collect()
    }

    #[test]
    fn test_full_pass_downloads_everything_and_reports_new_pair() {
        let (codebook, transport, _cache) = codebook_with(all_ok(8));
        let mut backoff = Backoff::default();

        assert!(run_pass(&codebook, &mut backoff, Instant::now()));
        assert_eq!(transport.requests().len(), 8);

        // Everything fresh: a second pass makes no requests (empty script
        // would panic) and reports nothing new.
        assert!(!run_pass(&codebook, &mut backoff, Instant::now()));
        assert_eq!(transport.requests().len(), 8);
    }

    #[test]
    fn test_permanent_failures_are_never_retried() {
        // Every fetch 404s: 6 text lists + 1 for en_us (aff fails first)
        let script = (0..7).map(|_| ok(404, "", None)).collect();
        let (codebook, transport, _cache) = codebook_with(script);
        let mut backoff = Backoff::default();

        assert!(!run_pass(&codebook, &mut backoff, Instant::now()));
        assert_eq!(transport.requests().len(), 7);

        // No retries — not even after a kick-style reset
        backoff.reset_transient();
        run_pass(&codebook, &mut backoff, Instant::now());
        assert_eq!(transport.requests().len(), 7);
        assert!(backoff.next_retry_delay(Instant::now()).is_none());
    }

    #[test]
    fn test_transient_failures_back_off_and_recover() {
        let script = (0..7).map(|_| connection_error()).collect();
        let (codebook, transport, _cache) = codebook_with(script);
        let mut backoff = Backoff::default();
        let now = Instant::now();

        assert!(!run_pass(&codebook, &mut backoff, now));
        assert_eq!(transport.requests().len(), 7);
        assert_eq!(backoff.next_retry_delay(now), Some(INITIAL_RETRY_DELAY));

        // Still backing off: nothing is attempted
        run_pass(&codebook, &mut backoff, now);
        assert_eq!(transport.requests().len(), 7);

        // After the delay elapses, the ids are retried and succeed
        for _ in 0..8 {
            transport.push(ok(200, "word\n", None));
        }
        let later = now + INITIAL_RETRY_DELAY;
        assert!(run_pass(&codebook, &mut backoff, later));
        assert_eq!(transport.requests().len(), 15);
    }

    #[test]
    fn test_kick_resets_transient_backoff() {
        let script = (0..7).map(|_| connection_error()).collect();
        let (codebook, transport, _cache) = codebook_with(script);
        let mut backoff = Backoff::default();
        let now = Instant::now();

        run_pass(&codebook, &mut backoff, now);
        assert_eq!(transport.requests().len(), 7);

        // A kick clears the backoff so the very next pass retries
        backoff.reset_transient();
        for _ in 0..8 {
            transport.push(ok(200, "word\n", None));
        }
        assert!(run_pass(&codebook, &mut backoff, now));
        assert_eq!(transport.requests().len(), 15);
    }

    #[test]
    fn test_backoff_delay_grows_exponentially_to_cap() {
        let mut backoff = Backoff::default();
        let now = Instant::now();

        assert_eq!(backoff.record_failure("id", now), Duration::from_secs(30));
        assert_eq!(backoff.record_failure("id", now), Duration::from_secs(60));
        assert_eq!(backoff.record_failure("id", now), Duration::from_secs(120));
        for _ in 0..10 {
            backoff.record_failure("id", now);
        }
        assert_eq!(backoff.record_failure("id", now), MAX_RETRY_DELAY);

        assert!(!backoff.ready("id", now));
        assert!(backoff.ready("id", now + MAX_RETRY_DELAY));
        assert!(backoff.ready("other", now));
    }

    #[test]
    fn test_next_retry_delay_is_min_over_transients() {
        let mut backoff = Backoff::default();
        let now = Instant::now();
        backoff.record_failure("a", now); // 30s
        backoff.record_failure("b", now); // 30s
        backoff.record_failure("b", now); // 60s
        assert_eq!(backoff.next_retry_delay(now), Some(Duration::from_secs(30)));
        // Past both deadlines the delay saturates to zero
        assert_eq!(
            backoff.next_retry_delay(now + Duration::from_secs(120)),
            Some(Duration::ZERO)
        );
    }

    #[tokio::test]
    async fn test_spawned_worker_emits_event_for_new_pair() {
        let (codebook, _transport, _cache) = codebook_with(all_ok(8));
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = spawn(Arc::new(codebook), tx);
        assert!(
            rx.recv().await.is_some(),
            "worker exited without emitting an event"
        );
        drop(handle); // disconnects the command channel; the worker exits
    }
}
