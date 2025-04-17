pub mod dictionaries;
mod logging;
pub mod parser;
pub mod queries;
mod splitter;

use std::sync::Arc;

use codebook_config::CodebookConfig;
use dictionaries::{dictionary, manager::DictionaryManager};
use dictionary::Dictionary;
use log::debug;
use parser::WordLocation;

pub struct Codebook {
    config: Arc<CodebookConfig>,
    manager: DictionaryManager,
}

// Custom 'codebook' dictionary could be removed later for a more general solution.
static DEFAULT_DICTIONARIES: &[&str; 3] = &["codebook", "software_terms", "computing_acronyms"];

impl Codebook {
    pub fn new(config: Arc<CodebookConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let manager = DictionaryManager::new(&config.cache_dir);
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
        if file_path.is_some() && self.config.should_ignore_path(file_path.unwrap()) {
            return Vec::new();
        }
        // get needed dictionary names
        // get needed dictionaries
        // call spell check on each dictionary
        let language = self.resolve_language(language, file_path);
        let dictionaries = self.get_dictionaries(Some(language));
        parser::find_locations(text, language, |word| {
            if self.config.should_flag_word(word) {
                return false;
            }
            if word.len() < 3 {
                return true;
            }
            if self.config.is_allowed_word(word) {
                return true;
            }
            for dictionary in &dictionaries {
                if dictionary.check(word) {
                    return true;
                }
            }
            false
        })
    }

    fn resolve_language(
        &self,
        language_type: Option<queries::LanguageType>,
        path: Option<&str>,
    ) -> queries::LanguageType {
        // Check if we have a language_id first, fallback to path, fall back to text
        match language_type {
            Some(lang) => lang,
            None => match path {
                Some(path) => queries::get_language_name_from_filename(path),
                None => queries::LanguageType::Text,
            },
        }
    }

    fn get_dictionaries(
        &self,
        language: Option<queries::LanguageType>,
    ) -> Vec<Arc<dyn Dictionary>> {
        let mut dictionary_ids = self.config.get_dictionary_ids();
        if let Some(lang) = language {
            let language_dictionary_ids = lang.dictionary_ids();
            dictionary_ids.extend(language_dictionary_ids);
        };
        dictionary_ids.extend(DEFAULT_DICTIONARIES.iter().map(|f| f.to_string()));
        let mut dictionaries = Vec::with_capacity(dictionary_ids.len());
        debug!("Checking text with dictionaries: {:?}", dictionary_ids);
        for dictionary_id in dictionary_ids {
            let dictionary = self.manager.get_dictionary(&dictionary_id);
            if let Some(d) = dictionary {
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
        // Get top suggestions and return the first 5 suggestions in round robin order
        let max_results = 5;
        let dictionaries = self.get_dictionaries(None);
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
            if let Some(item) = source.get(i) {
                if !result.contains(item) {
                    result.push(item.clone());
                    if result.len() >= max_count {
                        return result;
                    }
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
        // Round-robin order: first from each source, then second from each source
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

        // In round-robin, we get:
        // 1. apple (1st from 1st source)
        // 2. banana (1st from 2nd source) - cherry already taken
        // 3. cherry (1st from 3rd source)
        // 4. banana (2nd from 1st source)
        // 5. date (3rd from 2nd source) - cherry already taken
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

        // Round-robin order with uneven sources
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

        // Round-robin order, skipping empty source
        let result = collect_round_robin(&sources, 4);
        assert_eq!(result, vec!["apple", "cherry", "banana", "date"]);
    }

    #[test]
    fn test_collect_round_robin_with_numbers() {
        let sources = vec![vec![1, 3, 5], vec![2, 4, 6]];

        // Round-robin order with numbers
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

        // First round of round-robin (first from each source)
        let result = collect_round_robin(&sources, 3);
        assert_eq!(result, vec!["apple", "date", "grape"]);
    }

    #[test]
    fn test_collect_round_robin_max_count_higher_than_available() {
        let sources = vec![vec!["apple", "banana"], vec!["cherry", "date"]];

        // Round-robin order for all available elements
        let result = collect_round_robin(&sources, 10);
        assert_eq!(result, vec!["apple", "banana", "cherry", "date"]);
    }
}
