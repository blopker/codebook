pub mod checker;
pub mod dictionaries;
mod logging;
pub mod parser;
pub mod queries;
pub mod regexes;
pub mod regions;
mod splitter;

use crate::regexes::get_default_skip_patterns;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use codebook_config::CodebookConfig;
use dictionaries::{dictionary, manager::DictionaryManager};
use dictionary::Dictionary;
use log::debug;
use parser::WordLocation;

pub struct Codebook {
    config: Arc<dyn CodebookConfig>,
    manager: DictionaryManager,
}

// Custom 'codebook' dictionary could be removed later for a more general solution.
static DEFAULT_DICTIONARIES: &[&str; 3] = &["codebook", "software_terms", "computing_acronyms"];

impl Codebook {
    pub fn new(config: Arc<dyn CodebookConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = DictionaryManager::new(&config.cache_dir().to_path_buf());
        Ok(Self { config, manager })
    }

    /// Get WordLocations for a block of text.
    /// Supply LanguageType, file path or both to use the correct code parser.
    pub fn spell_check(
        &self,
        text: &str,
        language: Option<queries::LanguageType>,
        file_path: Option<&str>,
    ) -> Vec<parser::WordLocation> {
        if let Some(file_path) = file_path {
            // ignore_paths is a blocklist and has higher precedence than include_paths
            if self.config.should_ignore_path(Path::new(file_path)) {
                return Vec::new();
            }
            // include_paths is an allowlist; empty list means "include everything"
            if !self.config.should_include_path(Path::new(file_path)) {
                return Vec::new();
            }
        }

        let language = self.resolve_language(language, file_path);

        // Combine default and user skip patterns
        let mut all_patterns = get_default_skip_patterns().clone();
        if let Some(user_patterns) = self.config.get_ignore_patterns() {
            all_patterns.extend(user_patterns);
        }

        // Stage 1: Split into language regions
        let text_regions = regions::extract_regions(text, language);

        // Collect dictionaries for all languages present in the file
        let dictionaries = self.get_dictionaries_for_languages(&text_regions);

        // Stages 2+3: Extract nodes and words from each region
        let mut all_candidates = Vec::new();
        for region in &text_regions {
            // Stage 2: Node extraction
            let nodes =
                parser::extract_nodes(text, region, &|tag| self.config.should_check_tag(tag));
            // Stage 3: Word extraction
            let candidates = parser::extract_words(text, &nodes, &all_patterns);
            all_candidates.extend(candidates);
        }

        // Stage 4: Word checking
        checker::check_words(&all_candidates, &dictionaries, self.config.as_ref())
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

    /// Gather dictionaries for all languages present in a file.
    fn get_dictionaries_for_languages(
        &self,
        regions: &[regions::TextRegion],
    ) -> Vec<Arc<dyn Dictionary>> {
        let mut dictionary_ids = self.config.get_dictionary_ids();

        // Add language-specific dictionaries for all languages in the file
        let mut seen_languages = HashSet::new();
        for region in regions {
            if seen_languages.insert(region.language) {
                dictionary_ids.extend(region.language.dictionary_ids());
            }
        }

        // Add defaults
        dictionary_ids.extend(DEFAULT_DICTIONARIES.iter().map(|f| f.to_string()));

        // Deduplicate
        dictionary_ids.sort();
        dictionary_ids.dedup();

        let mut dictionaries = Vec::with_capacity(dictionary_ids.len());
        debug!("Checking text with dictionaries: {dictionary_ids:?}");
        for dictionary_id in dictionary_ids {
            if let Some(d) = self.manager.get_dictionary(&dictionary_id) {
                dictionaries.push(d);
            }
        }
        dictionaries
    }

    pub fn spell_check_file(&self, path: &str) -> Vec<WordLocation> {
        let lang_type = queries::get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path).unwrap();
        self.spell_check(&file_text, Some(lang_type), Some(path))
    }

    pub fn get_suggestions(&self, word: &str) -> Option<Vec<String>> {
        let max_results = 5;
        let dictionaries = self.get_dictionaries_for_languages(&[]);
        let mut is_misspelled = false;
        let suggestions: Vec<Vec<String>> = dictionaries
            .iter()
            .filter_map(|dict| {
                if !dict.check(word) {
                    is_misspelled = true;
                    Some(dict.suggest(word))
                } else {
                    None
                }
            })
            .collect();
        if !is_misspelled {
            return None;
        }
        Some(collect_round_robin(&suggestions, max_results))
    }
}

fn collect_round_robin<T: Clone + PartialEq + Ord>(sources: &[Vec<T>], max_count: usize) -> Vec<T> {
    let mut result = Vec::with_capacity(max_count);
    for i in 0..max_count {
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
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(result, vec!["apple", "banana", "cherry", "date"]);
    }
}
