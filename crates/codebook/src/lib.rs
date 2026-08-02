pub mod checker;
pub mod dictionaries;
mod logging;
pub mod parser;
pub mod queries;
pub mod regexes;
mod splitter;

use crate::regexes::get_default_skip_patterns;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use codebook_config::{CodebookConfig, ConfigSettings};
use dictionaries::{dictionary, manager::DictionaryManager};
use dictionary::Dictionary;
use log::debug;
use parser::WordLocation;

pub use codebook_downloader::network_disabled;
pub use dictionaries::manager::{EnsureError, EnsureOutcome};

/// Result of a blocking [`Codebook::warm_dictionaries`] pass.
pub struct WarmupReport {
    /// NO_NETWORK was set; nothing was fetched.
    pub network_disabled: bool,
    pub failures: Vec<(String, EnsureError)>,
    /// See [`Codebook::primary_dictionary_available`].
    pub primary_available: bool,
}

pub struct Codebook {
    config: Arc<dyn CodebookConfig>,
    manager: DictionaryManager,
}

// Custom 'codebook' dictionary could be removed later for a more general solution.
pub static DEFAULT_DICTIONARIES: &[&str; 3] = &["codebook", "software_terms", "computing_acronyms"];

impl Codebook {
    pub fn new(config: Arc<dyn CodebookConfig>) -> Self {
        Self::with_dictionary_dir(config, None)
    }

    /// Like `new`, but dictionaries are resolved from `dictionary_dir`
    /// (`{id}.txt` word lists or `{id}.aff` + `{id}.dic` Hunspell pairs)
    /// before falling back to downloading. Tests use this with checked-in
    /// fixtures so they never touch the network.
    pub fn with_dictionary_dir(
        config: Arc<dyn CodebookConfig>,
        dictionary_dir: Option<std::path::PathBuf>,
    ) -> Self {
        let manager =
            DictionaryManager::with_local_dir(&config.cache_dir().to_path_buf(), dictionary_dir);
        Self { config, manager }
    }

    /// Create a Codebook from a preconstructed manager. Tests use this with
    /// `DictionaryManager::with_transport` to exercise download flows
    /// without sockets.
    pub fn with_manager(config: Arc<dyn CodebookConfig>, manager: DictionaryManager) -> Self {
        Self { config, manager }
    }

    /// Blocking: download or revalidate one dictionary. Used by the LSP's
    /// background prefetch worker and the CLI's synchronous warmup — never on
    /// the spell-check path.
    pub fn ensure_dictionary(&self, id: &str) -> Result<EnsureOutcome, EnsureError> {
        self.manager.ensure_dictionary(id)
    }

    /// True when at least one configured dictionary that can serve as a
    /// primary (natural-language) dictionary is loadable. When this is
    /// false every check noops, so one-shot callers like the CLI should
    /// treat it as a failure rather than report a clean run.
    pub fn primary_dictionary_available(&self) -> bool {
        self.any_primary_available(&self.config.all_dictionary_ids())
    }

    /// The cold-cache noop rule, in one place: some primary-capable
    /// configured dictionary must be loadable. A configuration with no
    /// primary-capable id at all is taken at face value — any loadable
    /// configured dictionary counts.
    fn any_primary_available(&self, ids: &[String]) -> bool {
        let has_primary_capable = ids.iter().any(|id| self.manager.is_primary_capable(id));
        ids.iter().any(|id| {
            (!has_primary_capable || self.manager.is_primary_capable(id))
                && self.manager.get_dictionary(id).is_some()
        })
    }

    /// Every dictionary id worth prefetching: any id the config can resolve
    /// to (including per-path overrides) plus every downloadable
    /// per-language word list (they're small, and a file of that language
    /// may first be opened while offline). The defaults need no separate
    /// mention: DEFAULT_DICTIONARIES are all text repos, and the implicit
    /// en_us is part of the config's ids.
    pub fn prefetch_dictionary_ids(&self) -> Vec<String> {
        let mut ids = self.config.all_dictionary_ids();
        ids.extend(dictionaries::repo::text_dictionary_ids().map(str::to_string));
        ids.sort();
        ids.dedup();
        ids
    }

    /// Blocking: download or revalidate every prefetchable dictionary, for
    /// one-shot callers (the lint CLI, the dev binary) that have no
    /// background worker. Reporting severity is the caller's business;
    /// `primary_available == false` means every check will noop.
    pub fn warm_dictionaries(&self) -> WarmupReport {
        let network_disabled = network_disabled();
        let mut failures = Vec::new();
        if !network_disabled {
            for id in self.prefetch_dictionary_ids() {
                if let Err(e) = self.ensure_dictionary(&id) {
                    failures.push((id, e));
                }
            }
        }
        WarmupReport {
            network_disabled,
            failures,
            primary_available: self.primary_dictionary_available(),
        }
    }

    /// Get WordLocations for a block of text.
    /// Supply LanguageType, file path or both to use the correct code parser.
    pub fn spell_check(
        &self,
        text: &str,
        language: Option<queries::LanguageType>,
        file_path: Option<&str>,
    ) -> Vec<parser::WordLocation> {
        // ignore_paths and include_paths are evaluated BEFORE overrides
        if let Some(file_path) = file_path {
            if self.config.should_ignore_path(Path::new(file_path)) {
                return Vec::new();
            }
            if !self.config.should_include_path(Path::new(file_path)) {
                return Vec::new();
            }
        }

        // Resolve per-file settings (applies matching overrides)
        let resolved = file_path.and_then(|fp| self.config.resolve_for_file(Path::new(fp)));

        let language = self.resolve_language(language, file_path);

        // Combine default and user skip patterns
        let mut all_patterns = get_default_skip_patterns().clone();
        if let Some(ref settings) = resolved {
            all_patterns.extend(settings.ignore_patterns.iter().cloned());
        } else {
            all_patterns.extend(self.config.get_ignore_patterns());
        }

        // Extract all words, recursively following injections
        let (candidates, languages_found) = parser::extract_all_words(
            text,
            language,
            &|tag| self.config.should_check_tag(tag),
            &all_patterns,
        );

        // Load dictionaries for all languages encountered (using resolved settings if any)
        let Some(dictionaries) =
            self.get_dictionaries_for_languages(&languages_found, resolved.as_deref())
        else {
            return Vec::new();
        };

        // Check words against dictionaries
        checker::check_words(
            &candidates,
            &dictionaries,
            self.config.as_ref(),
            resolved.as_deref(),
        )
    }

    fn resolve_language(
        &self,
        language_type: Option<queries::LanguageType>,
        path: Option<&str>,
    ) -> queries::LanguageType {
        match language_type {
            Some(lang) => lang,
            None => match path {
                Some(path) => queries::get_language_name_from_filename(path),
                None => queries::LanguageType::Text,
            },
        }
    }

    /// Gather dictionaries for all languages encountered in a file.
    /// If `resolved` is Some, its dictionary list is used in place of the base config's.
    ///
    /// Returns None when the noop rule fails (see `any_primary_available`) —
    /// e.g. a cold cache before the background download lands. Checking with
    /// only the supplementary word lists (the embedded `codebook` list is
    /// always present) would flag nearly every word, so callers must noop
    /// instead.
    fn get_dictionaries_for_languages(
        &self,
        languages: &HashSet<queries::LanguageType>,
        resolved: Option<&ConfigSettings>,
    ) -> Option<Vec<Arc<dyn Dictionary>>> {
        let mut dictionary_ids = match resolved {
            Some(settings) => settings.dictionary_ids(),
            None => self.config.get_dictionary_ids(),
        };

        if !self.any_primary_available(&dictionary_ids) {
            debug!("No configured primary dictionary available yet; skipping check");
            return None;
        }

        for lang in languages {
            dictionary_ids.extend(lang.dictionary_ids());
        }

        dictionary_ids.extend(DEFAULT_DICTIONARIES.iter().map(|f| f.to_string()));

        dictionary_ids.sort();
        dictionary_ids.dedup();

        let mut dictionaries = Vec::with_capacity(dictionary_ids.len());
        debug!("Checking text with dictionaries: {dictionary_ids:?}");
        for dictionary_id in dictionary_ids {
            if let Some(d) = self.manager.get_dictionary(&dictionary_id) {
                dictionaries.push(d);
            }
        }
        Some(dictionaries)
    }

    /// Spell check a file on disk, detecting the language from its path.
    /// Errors if the file can't be read (including non-UTF-8 content).
    pub fn spell_check_file(&self, path: &str) -> Result<Vec<WordLocation>, std::io::Error> {
        let lang_type = queries::get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path)?;
        Ok(self.spell_check(&file_text, Some(lang_type), Some(path)))
    }

    /// Get suggestions for a misspelled word. Returns None when the word is
    /// correctly spelled — same rule as check_words: any dictionary knowing
    /// the word makes it correct.
    pub fn get_suggestions(&self, word: &str) -> Option<Vec<String>> {
        let max_results = 5;
        let dictionaries = self.get_dictionaries_for_languages(&HashSet::new(), None)?;
        if dictionaries.is_empty() || dictionaries.iter().any(|dict| dict.check(word)) {
            return None;
        }
        let suggestions: Vec<Vec<String>> =
            dictionaries.iter().map(|dict| dict.suggest(word)).collect();
        Some(collect_round_robin(&suggestions, max_results))
    }
}

/// Interleave suggestion lists, preserving each source's ranking: every
/// source's best suggestion comes before any source's second-best.
fn collect_round_robin<T: Clone + PartialEq>(sources: &[Vec<T>], max_count: usize) -> Vec<T> {
    let mut result = Vec::with_capacity(max_count);
    let longest = sources.iter().map(Vec::len).max().unwrap_or(0);
    for i in 0..longest {
        for source in sources {
            if let Some(item) = source.get(i)
                && !result.contains(item)
            {
                result.push(item.clone());
                if result.len() >= max_count {
                    return result;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cold cache must noop rather than check against only the
    /// supplementary word lists — that would flag nearly every word.
    #[test]
    fn test_cold_cache_noops_until_configured_dictionary_arrives() {
        use codebook_downloader::testing::{FakeTransport, ok};

        let transport = FakeTransport::new(vec![]);
        let cache_dir = tempfile::tempdir().unwrap();
        let manager = dictionaries::manager::DictionaryManager::with_transport(
            &cache_dir.path().to_path_buf(),
            None,
            transport.clone(),
        );
        let config = Arc::new(codebook_config::CodebookConfigMemory::default());
        let codebook = Codebook::with_manager(config, manager);

        // en_us (the implicit default) isn't downloaded: no diagnostics, no
        // suggestions, no network (the empty script panics on any request).
        assert!(codebook.spell_check("mispeled wrods", None, None).is_empty());
        assert!(codebook.get_suggestions("mispeled").is_none());

        // Once the configured dictionary lands, checking resumes.
        transport.push(ok(200, "SET UTF-8\n", None));
        transport.push(ok(200, "1\nhello\n", None));
        assert_eq!(
            codebook.ensure_dictionary("en_us").unwrap(),
            EnsureOutcome::NewHunspellPair
        );
        let results = codebook.spell_check("hello wrold", None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "wrold");
    }

    fn codebook_with_config(
        settings: codebook_config::ConfigSettings,
        cache_dir: &std::path::Path,
        transport: Arc<codebook_downloader::testing::FakeTransport>,
    ) -> Codebook {
        let manager = dictionaries::manager::DictionaryManager::with_transport(
            &cache_dir.to_path_buf(),
            None,
            transport,
        );
        let config = Arc::new(codebook_config::CodebookConfigMemory::new(settings));
        Codebook::with_manager(config, manager)
    }

    /// Supplementary word lists (e.g. folded in via an override's
    /// extra_dictionaries) must not satisfy the cold-cache noop rule; only a
    /// primary-capable dictionary does.
    #[test]
    fn test_supplementary_dictionary_does_not_satisfy_noop_rule() {
        use codebook_downloader::testing::{FakeTransport, ok};

        let settings = codebook_config::ConfigSettings {
            dictionaries: vec!["en_us".to_string(), "software_terms".to_string()],
            ..Default::default()
        };
        let transport = FakeTransport::new(vec![ok(200, "hello\n", None)]);
        let cache_dir = tempfile::tempdir().unwrap();
        let codebook = codebook_with_config(settings, cache_dir.path(), transport.clone());

        // Only the word list has landed: still noop
        assert_eq!(
            codebook.ensure_dictionary("software_terms").unwrap(),
            EnsureOutcome::Refreshed
        );
        assert!(!codebook.primary_dictionary_available());
        assert!(codebook.spell_check("hello wrold", None, None).is_empty());

        // The Hunspell dictionary lands: checking resumes
        transport.push(ok(200, "SET UTF-8\n", None));
        transport.push(ok(200, "1\nhello\n", None));
        assert_eq!(
            codebook.ensure_dictionary("en_us").unwrap(),
            EnsureOutcome::NewHunspellPair
        );
        assert!(codebook.primary_dictionary_available());
        let results = codebook.spell_check("hello wrold", None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "wrold");
    }

    /// A config with no primary-capable id at all is taken at face value:
    /// its word lists alone enable checking.
    #[test]
    fn test_config_with_only_word_lists_still_checks() {
        use codebook_downloader::testing::{FakeTransport, ok};

        let settings = codebook_config::ConfigSettings {
            dictionaries: vec!["software_terms".to_string()],
            ..Default::default()
        };
        let transport = FakeTransport::new(vec![ok(200, "hello\n", None)]);
        let cache_dir = tempfile::tempdir().unwrap();
        let codebook = codebook_with_config(settings, cache_dir.path(), transport.clone());

        codebook.ensure_dictionary("software_terms").unwrap();
        assert!(codebook.primary_dictionary_available());
        let results = codebook.spell_check("hello qzwx", None, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "qzwx");
    }

    #[test]
    fn test_collect_round_robin_basic() {
        let sources = vec![
            vec!["apple", "banana", "cherry"],
            vec!["date", "elderberry", "fig"],
            vec!["grape", "honeydew", "kiwi"],
        ];
        let result = collect_round_robin(&sources, 5);
        assert_eq!(
            result,
            vec!["apple", "date", "grape", "banana", "elderberry"]
        );
    }

    #[test]
    fn test_collect_round_robin_with_duplicates() {
        let sources = vec![
            vec!["apple", "banana", "cherry"],
            vec!["banana", "cherry", "date"],
            vec!["cherry", "date", "elderberry"],
        ];
        let result = collect_round_robin(&sources, 5);
        assert_eq!(
            result,
            vec!["apple", "banana", "cherry", "date", "elderberry"]
        );
    }

    #[test]
    fn test_collect_round_robin_uneven_sources() {
        let sources = vec![
            vec!["apple", "banana", "cherry", "date"],
            vec!["elderberry"],
            vec!["fig", "grape"],
        ];
        let result = collect_round_robin(&sources, 7);
        assert_eq!(
            result,
            vec![
                "apple",
                "elderberry",
                "fig",
                "banana",
                "grape",
                "cherry",
                "date"
            ]
        );
    }

    #[test]
    fn test_collect_round_robin_empty_sources() {
        let sources: Vec<Vec<&str>> = vec![];
        let result = collect_round_robin(&sources, 5);
        assert_eq!(result, Vec::<&str>::new());
    }

    #[test]
    fn test_collect_round_robin_some_empty_sources() {
        let sources = vec![vec!["apple", "banana"], vec![], vec!["cherry", "date"]];
        let result = collect_round_robin(&sources, 4);
        assert_eq!(result, vec!["apple", "cherry", "banana", "date"]);
    }

    #[test]
    fn test_collect_round_robin_with_numbers() {
        let sources = vec![vec![1, 3, 5], vec![2, 4, 6]];
        let result = collect_round_robin(&sources, 6);
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_collect_round_robin_max_count_exceeded() {
        let sources = vec![
            vec!["apple", "banana", "cherry"],
            vec!["date", "elderberry", "fig"],
            vec!["grape", "honeydew", "kiwi"],
        ];
        let result = collect_round_robin(&sources, 3);
        assert_eq!(result, vec!["apple", "date", "grape"]);
    }

    #[test]
    fn test_collect_round_robin_max_count_higher_than_available() {
        let sources = vec![vec!["apple", "banana"], vec!["cherry", "date"]];
        let result = collect_round_robin(&sources, 10);
        // Round-robin order is preserved even when under the cap
        assert_eq!(result, vec!["apple", "cherry", "banana", "date"]);
    }
}
