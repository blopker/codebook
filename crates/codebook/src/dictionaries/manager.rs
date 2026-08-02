use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use super::{
    dictionary::{self, TextDictionary},
    repo::{DictionaryRepo, HunspellRepo, TextRepo, get_repo},
    transliteration::TransliteratingDictionary,
};
use codebook_downloader::{Downloader, FetchOutcome, PendingDownload, PermanentHttpError};
use dictionary::{Dictionary, HunspellDictionary};
use log::{debug, error, info};

/// Result of a blocking [`DictionaryManager::ensure_dictionary`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnsureOutcome {
    /// Nothing to fetch: the id has no repo entry, or the dictionary is
    /// embedded in the binary / provided by the local override directory.
    Unavailable,
    /// Already on disk and within the revalidation window.
    Fresh,
    /// Downloaded a Hunspell pair that wasn't loadable before. (Which
    /// outcomes trigger re-checking open documents is the LSP worker's
    /// policy, not this crate's.)
    NewHunspellPair,
    /// Committed new content: a refreshed Hunspell pair or a text word list
    /// (new or updated).
    Refreshed,
}

/// A dictionary fetch failure. Permanent failures are not worth retrying
/// this process: the server definitively answered 4xx, or NO_NETWORK is set.
/// (Hand-written impls: thiserror can't derive over an `anyhow::Error`
/// source field.)
#[derive(Debug)]
pub struct EnsureError {
    pub permanent: bool,
    pub source: anyhow::Error,
}

impl EnsureError {
    fn from_download(source: anyhow::Error) -> Self {
        Self {
            permanent: source.downcast_ref::<PermanentHttpError>().is_some()
                || source
                    .downcast_ref::<codebook_downloader::NetworkDisabledError>()
                    .is_some(),
            source,
        }
    }
}

impl std::fmt::Display for EnsureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl std::error::Error for EnsureError {}

/// A user-provided dictionary in the local override directory.
enum LocalOverride {
    Text(PathBuf),
    Hunspell { aff: PathBuf, dic: PathBuf },
}

pub struct DictionaryManager {
    dictionary_cache: RwLock<HashMap<String, Arc<dyn Dictionary>>>,
    /// Bumped by every on-disk invalidation. A `get_dictionary` load that
    /// straddles a commit could otherwise insert a dictionary built from the
    /// old files *after* the invalidation removed it, pinning stale content
    /// in memory until the next revalidation; loads only cache when the
    /// epoch is unchanged since they started. Also validates
    /// `missing_since`: negative entries expire the moment anything commits.
    invalidation_epoch: AtomicU64,
    /// Ids that had no loadable files, keyed to the epoch of that probe.
    /// Spares the per-keystroke check path from re-statting the disk for
    /// every missing dictionary; self-invalidates when the epoch moves.
    missing_since: RwLock<HashMap<String, u64>>,
    not_available_logged: Mutex<HashSet<String>>,
    downloader: Downloader,
    local_dir: Option<PathBuf>,
}

impl DictionaryManager {
    pub fn new(cache_dir: &PathBuf) -> Self {
        Self::with_local_dir(cache_dir, None)
    }

    /// Create a manager that resolves dictionaries from a local directory of
    /// `{id}.txt` word lists or `{id}.aff` + `{id}.dic` Hunspell pairs before
    /// falling back to the download repo. Tests use this with checked-in
    /// fixtures so `cargo test` never touches the network.
    pub fn with_local_dir(cache_dir: &PathBuf, local_dir: Option<PathBuf>) -> Self {
        Self::from_downloader(Downloader::new(cache_dir), local_dir)
    }

    /// Create a manager with a custom HTTP transport. Tests use this to
    /// exercise the download paths without sockets.
    pub fn with_transport(
        cache_dir: &PathBuf,
        local_dir: Option<PathBuf>,
        transport: Arc<dyn codebook_downloader::HttpTransport>,
    ) -> Self {
        Self::from_downloader(Downloader::with_transport(cache_dir, transport), local_dir)
    }

    fn from_downloader(downloader: Downloader, local_dir: Option<PathBuf>) -> Self {
        Self {
            dictionary_cache: RwLock::new(HashMap::new()),
            invalidation_epoch: AtomicU64::new(0),
            missing_since: RwLock::new(HashMap::new()),
            not_available_logged: Mutex::new(HashSet::new()),
            downloader,
            local_dir,
        }
    }

    /// Whether the id can serve as a primary (natural-language) dictionary:
    /// a Hunspell repo or a user-provided local override. Supplementary word
    /// lists must not count — the noop rule in `Codebook` depends on it.
    pub fn is_primary_capable(&self, id: &str) -> bool {
        self.local_override(id).is_some() || super::repo::is_hunspell(id)
    }

    /// Blocking: download or revalidate the dictionary's files. Multi-file
    /// (aff/dic) downloads are staged first and committed all-or-nothing, so
    /// a failure part-way can't leave a mismatched pair on disk. Never called
    /// on the spell-check path — the LSP's background prefetch worker and the
    /// CLI's synchronous warmup use it.
    pub fn ensure_dictionary(&self, id: &str) -> Result<EnsureOutcome, EnsureError> {
        if self.local_override(id).is_some() {
            return Ok(EnsureOutcome::Unavailable);
        }
        match get_repo(id) {
            Some(DictionaryRepo::Hunspell(r)) => self.ensure_hunspell(id, &r),
            Some(DictionaryRepo::Text(r)) => self.ensure_text(id, &r),
            None => Ok(EnsureOutcome::Unavailable),
        }
    }

    /// The single home of the local-override file convention:
    /// `{id}.txt`, or `{id}.aff` + `{id}.dic`.
    fn local_override(&self, id: &str) -> Option<LocalOverride> {
        let dir = self.local_dir.as_ref()?;
        let txt = dir.join(format!("{id}.txt"));
        if txt.is_file() {
            return Some(LocalOverride::Text(txt));
        }
        let aff = dir.join(format!("{id}.aff"));
        let dic = dir.join(format!("{id}.dic"));
        (aff.is_file() && dic.is_file()).then_some(LocalOverride::Hunspell { aff, dic })
    }

    fn ensure_hunspell(&self, id: &str, repo: &HunspellRepo) -> Result<EnsureOutcome, EnsureError> {
        let was_loadable = self.downloader.local_path(&repo.aff_url).is_some()
            && self.downloader.local_path(&repo.dict_url).is_some();

        // Stage both files before committing either, so a fetch failure
        // part-way can't leave a mismatched pair on disk.
        let mut aff = self.fetch(&repo.aff_url, false)?;
        let mut dic = self.fetch(&repo.dict_url, false)?;

        // The pair must move between generations together. When only one
        // half changed, the other's revalidation clock may simply have
        // diverged (e.g. its last check failed) — force-revalidate it so an
        // upstream update of both files can't be applied one-sided.
        let aff_pending = matches!(aff, FetchOutcome::Pending(_));
        let dic_pending = matches!(dic, FetchOutcome::Pending(_));
        if aff_pending && !dic_pending {
            dic = self.fetch(&repo.dict_url, true)?;
        } else if dic_pending && !aff_pending {
            aff = self.fetch(&repo.aff_url, true)?;
        }

        let committed = matches!(aff, FetchOutcome::Pending(_))
            || matches!(dic, FetchOutcome::Pending(_));
        for outcome in [aff, dic] {
            if let FetchOutcome::Pending(pending) = outcome {
                self.commit(pending)?;
            }
        }
        if !committed {
            return Ok(EnsureOutcome::Fresh);
        }
        // The files changed; drop any dictionary loaded from the old ones.
        self.invalidate(id);
        Ok(if was_loadable {
            EnsureOutcome::Refreshed
        } else {
            EnsureOutcome::NewHunspellPair
        })
    }

    fn ensure_text(&self, id: &str, repo: &TextRepo) -> Result<EnsureOutcome, EnsureError> {
        if repo.text.is_some() {
            return Ok(EnsureOutcome::Unavailable);
        }
        let Some(url) = repo.url.as_ref() else {
            return Ok(EnsureOutcome::Unavailable);
        };
        match self.fetch(url, false)? {
            FetchOutcome::Pending(pending) => {
                self.commit(pending)?;
                self.invalidate(id);
                Ok(EnsureOutcome::Refreshed)
            }
            FetchOutcome::UpToDate(_) => Ok(EnsureOutcome::Fresh),
        }
    }

    fn fetch(&self, url: &str, force: bool) -> Result<FetchOutcome, EnsureError> {
        self.downloader
            .fetch(url, force)
            .map_err(EnsureError::from_download)
    }

    fn commit(&self, pending: PendingDownload) -> Result<PathBuf, EnsureError> {
        self.downloader
            .commit(pending)
            .map_err(EnsureError::from_download)
    }

    /// Resolve a dictionary from memory, the local override directory, the
    /// embedded word lists, or the on-disk download cache — never the
    /// network. Downloading happens through [`Self::ensure_dictionary`], off
    /// the spell-check path; until it lands, missing dictionaries return
    /// `None` and are simply skipped. A stale-but-present copy still loads —
    /// revalidation is the prefetch worker's job.
    pub fn get_dictionary(&self, id: &str) -> Option<Arc<dyn Dictionary>> {
        {
            let cache = self.dictionary_cache.read().unwrap();
            if let Some(dictionary) = cache.get(id) {
                return Some(dictionary.clone());
            }
        }

        let epoch = self.invalidation_epoch.load(Ordering::Acquire);

        if let Some(d) = self.get_local_dictionary(id) {
            self.cache_unless_invalidated(id, &d, epoch);
            return Some(d);
        }

        // Negative cache: skip the repo scan and disk probes when this id
        // already came up missing and nothing has committed since. (Checked
        // after the local-dir probe so dropping a file there is still picked
        // up immediately; local_dir is None outside tests.)
        if self.missing_since.read().unwrap().get(id) == Some(&epoch) {
            return None;
        }

        let repo = match get_repo(id) {
            Some(r) => r,
            None => {
                debug!("Failed to get repo for dictionary, skipping: {id}");
                return None;
            }
        };

        let dictionary = match repo {
            DictionaryRepo::Hunspell(r) => self.load_hunspell_from_disk(&r),
            DictionaryRepo::Text(r) => self.load_text_from_disk(&r),
        };

        match dictionary {
            Some(d) => {
                self.cache_unless_invalidated(id, &d, epoch);
                Some(d)
            }
            None => {
                self.missing_since
                    .write()
                    .unwrap()
                    .insert(id.to_string(), epoch);
                self.log_not_available(id);
                None
            }
        }
    }

    /// Insert into the in-memory cache only when no invalidation happened
    /// while the dictionary was being read from disk. A load racing a commit
    /// may have seen the old files (or a mixed aff/dic pair mid-swap);
    /// caching it would outlive the invalidation and pin stale content until
    /// the next revalidation. Skipping the insert just means the next call
    /// reloads from the (new) files.
    fn cache_unless_invalidated(&self, id: &str, dictionary: &Arc<dyn Dictionary>, epoch: u64) {
        let mut cache = self.dictionary_cache.write().unwrap();
        if self.invalidation_epoch.load(Ordering::Acquire) == epoch {
            cache.insert(id.to_string(), dictionary.clone());
        }
    }

    /// Drop the in-memory dictionary after its on-disk files changed.
    fn invalidate(&self, id: &str) {
        // Bump before removing: a load already past its epoch read either
        // sees the remove happen after its insert (covered by the remove
        // below) or fails the epoch check in cache_unless_invalidated.
        self.invalidation_epoch.fetch_add(1, Ordering::Release);
        self.dictionary_cache.write().unwrap().remove(id);
    }

    /// Once per id per process, so a cold cache is diagnosable without
    /// logging on every check.
    fn log_not_available(&self, id: &str) {
        if self.not_available_logged.lock().unwrap().insert(id.to_string()) {
            info!(
                "Dictionary '{id}' is not available locally yet; checks skip it until a background download completes"
            );
        }
    }

    /// Load a dictionary from the local override directory, if configured.
    fn get_local_dictionary(&self, id: &str) -> Option<Arc<dyn Dictionary>> {
        match self.local_override(id)? {
            LocalOverride::Text(txt) => Some(Arc::new(TextDictionary::new_from_path(&txt))),
            LocalOverride::Hunspell { aff, dic } => {
                match HunspellDictionary::new(aff.to_str()?, dic.to_str()?) {
                    Ok(dict) => Some(Arc::new(dict)),
                    Err(e) => {
                        error!("Failed to load local dictionary '{id}': {e}");
                        None
                    }
                }
            }
        }
    }

    fn load_hunspell_from_disk(&self, repo: &HunspellRepo) -> Option<Arc<dyn Dictionary>> {
        let aff_path = self.downloader.local_path(&repo.aff_url)?;
        let dic_path = self.downloader.local_path(&repo.dict_url)?;
        let (Some(aff), Some(dic)) = (aff_path.to_str(), dic_path.to_str()) else {
            error!("Dictionary cache path is not valid UTF-8: {aff_path:?}");
            return None;
        };
        let dict = match HunspellDictionary::new(aff, dic) {
            Ok(dict) => dict,
            Err(e) => {
                error!("Failed to load Hunspell dictionary: {e}");
                return None;
            }
        };
        let base: Arc<dyn Dictionary> = Arc::new(dict);
        Some(match repo.transliteration {
            Some(t) => Arc::new(TransliteratingDictionary::new(base, t.variants_fn())),
            None => base,
        })
    }

    fn load_text_from_disk(&self, repo: &TextRepo) -> Option<Arc<dyn Dictionary>> {
        if let Some(text) = repo.text {
            return Some(Arc::new(TextDictionary::new(text)));
        }
        let path = self.downloader.local_path(repo.url.as_ref()?)?;
        Some(Arc::new(TextDictionary::new_from_path(&path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codebook_downloader::testing::{FakeTransport, ok};
    use tempfile::tempdir;

    const AFF: &str = "SET UTF-8\n";
    const DIC: &str = "1\nhello\n";

    fn hunspell_urls(id: &str) -> (String, String) {
        match get_repo(id) {
            Some(DictionaryRepo::Hunspell(r)) => (r.aff_url, r.dict_url),
            other => panic!("expected Hunspell repo for {id}, got {other:?}"),
        }
    }

    fn text_url(id: &str) -> String {
        match get_repo(id) {
            Some(DictionaryRepo::Text(r)) => r.url.unwrap(),
            other => panic!("expected Text repo for {id}, got {other:?}"),
        }
    }

    fn manager_with(transport: Arc<FakeTransport>, dir: &std::path::Path) -> DictionaryManager {
        DictionaryManager::with_transport(&dir.to_path_buf(), None, transport)
    }

    #[test]
    fn test_get_dictionary_cold_cache_makes_no_requests() {
        // The empty script panics on any request: the check path must never
        // touch the network.
        let transport = FakeTransport::new(vec![]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport, dir.path());

        assert!(manager.get_dictionary("en_us").is_none());
        assert!(manager.get_dictionary("rust").is_none());
        // Embedded dictionary still resolves offline
        assert!(manager.get_dictionary("codebook").is_some());
    }

    #[test]
    fn test_ensure_downloads_new_hunspell_pair() {
        let (aff_url, dic_url) = hunspell_urls("en_us");
        let transport = FakeTransport::new(vec![ok(200, AFF, None), ok(200, DIC, None)]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport.clone(), dir.path());

        let outcome = manager.ensure_dictionary("en_us").unwrap();
        assert_eq!(outcome, EnsureOutcome::NewHunspellPair);
        let requests = transport.requests();
        assert_eq!(requests[0].0, aff_url);
        assert_eq!(requests[1].0, dic_url);

        // Loads from disk without further requests (empty script would panic)
        let dict = manager.get_dictionary("en_us").expect("dictionary loads");
        assert!(dict.check("hello"));

        // A second ensure within the revalidation window is a no-op
        assert_eq!(
            manager.ensure_dictionary("en_us").unwrap(),
            EnsureOutcome::Fresh
        );
    }

    #[test]
    fn test_ensure_hunspell_commits_all_or_nothing() {
        let (aff_url, dic_url) = hunspell_urls("en_us");
        let transport = FakeTransport::new(vec![ok(200, AFF, None), ok(500, "", None)]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport.clone(), dir.path());

        let err = manager.ensure_dictionary("en_us").unwrap_err();
        assert!(!err.permanent);
        // The aff download succeeded but must not have been committed
        assert!(manager.downloader.local_path(&aff_url).is_none());
        assert!(manager.downloader.local_path(&dic_url).is_none());

        // A retry still reports a NEW pair, proving nothing was on disk
        transport.push(ok(200, AFF, None));
        transport.push(ok(200, DIC, None));
        assert_eq!(
            manager.ensure_dictionary("en_us").unwrap(),
            EnsureOutcome::NewHunspellPair
        );
    }

    #[test]
    fn test_ensure_404_is_permanent() {
        let transport = FakeTransport::new(vec![ok(404, "", None)]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport.clone(), dir.path());

        let err = manager.ensure_dictionary("en_us").unwrap_err();
        assert!(err.permanent);
        assert_eq!(transport.requests().len(), 1);
    }

    #[test]
    fn test_ensure_text_refresh_invalidates_loaded_dictionary() {
        let url = text_url("rust");
        let transport = FakeTransport::new(vec![ok(200, "hello\n", None)]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport.clone(), dir.path());

        assert_eq!(
            manager.ensure_dictionary("rust").unwrap(),
            EnsureOutcome::Refreshed
        );
        let dict = manager.get_dictionary("rust").unwrap();
        assert!(dict.check("hello"));
        assert!(!dict.check("world"));

        // Age the entry and serve changed content: the in-memory dictionary
        // must be dropped so the next get sees the new word list.
        manager.downloader.force_stale(&url);
        transport.push(ok(200, "world\n", None));
        assert_eq!(
            manager.ensure_dictionary("rust").unwrap(),
            EnsureOutcome::Refreshed
        );
        let dict = manager.get_dictionary("rust").unwrap();
        assert!(dict.check("world"));
        assert!(!dict.check("hello"));
    }

    #[test]
    fn test_ensure_hunspell_moves_both_halves_between_generations() {
        let (aff_url, dic_url) = hunspell_urls("en_us");
        let transport = FakeTransport::new(vec![ok(200, AFF, None), ok(200, DIC, None)]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport.clone(), dir.path());
        manager.ensure_dictionary("en_us").unwrap();

        // Only aff's revalidation clock lapses (as if dic's last check
        // failed), while upstream has updated BOTH halves. The fresh dic
        // must be force-revalidated so the pair can't commit one-sided.
        manager.downloader.force_stale(&aff_url);
        transport.push(ok(200, "SET UTF-8\nTRY abc\n", None));
        transport.push(ok(200, "1\nworld\n", None));
        assert_eq!(
            manager.ensure_dictionary("en_us").unwrap(),
            EnsureOutcome::Refreshed
        );

        let aff = std::fs::read_to_string(manager.downloader.local_path(&aff_url).unwrap()).unwrap();
        let dic = std::fs::read_to_string(manager.downloader.local_path(&dic_url).unwrap()).unwrap();
        assert!(aff.contains("TRY"));
        assert!(dic.contains("world"));
        assert_eq!(transport.requests().len(), 4);
    }

    #[test]
    fn test_ensure_embedded_and_unknown_ids_are_unavailable() {
        let transport = FakeTransport::new(vec![]);
        let dir = tempdir().unwrap();
        let manager = manager_with(transport.clone(), dir.path());

        assert_eq!(
            manager.ensure_dictionary("codebook").unwrap(),
            EnsureOutcome::Unavailable
        );
        assert_eq!(
            manager.ensure_dictionary("no_such_dictionary").unwrap(),
            EnsureOutcome::Unavailable
        );
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn test_ensure_skips_local_override() {
        let transport = FakeTransport::new(vec![]);
        let cache_dir = tempdir().unwrap();
        let local_dir = tempdir().unwrap();
        std::fs::write(local_dir.path().join("en_us.txt"), "hello\n").unwrap();
        let manager = DictionaryManager::with_transport(
            &cache_dir.path().to_path_buf(),
            Some(local_dir.path().to_path_buf()),
            transport.clone(),
        );

        assert_eq!(
            manager.ensure_dictionary("en_us").unwrap(),
            EnsureOutcome::Unavailable
        );
        assert!(transport.requests().is_empty());
    }
}
