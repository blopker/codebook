use crate::dictionary_repo::get_codebook_dictionary;
use codebook_config::CodebookConfig;
use log::debug;
use lru::LruCache;

use std::{
    collections::HashSet,
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};

enum WordCase {
    AllCaps,
    AllLower,
    TitleCase,
    Unknown,
}

#[derive(Debug)]
pub struct CodeDictionary {
    custom_dictionary: Arc<RwLock<HashSet<String>>>,
    dictionary: spellbook::Dictionary,
    suggestion_cache: Arc<RwLock<LruCache<String, Vec<String>>>>,
    config: Arc<CodebookConfig>,
}

impl CodeDictionary {
    pub fn new(
        config: Arc<CodebookConfig>,
        aff_path: &str,
        dic_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let aff = std::fs::read_to_string(aff_path)?;
        let dic = std::fs::read_to_string(dic_path)?;
        let dict = spellbook::Dictionary::new(&aff, &dic)
            .map_err(|e| format!("Dictionary parse error: {}", e))?;
        let mut custom_dictionary: HashSet<String> = HashSet::new();
        for word in get_codebook_dictionary() {
            custom_dictionary.insert(word.to_string());
        }
        Ok(CodeDictionary {
            config,
            custom_dictionary: Arc::new(RwLock::new(custom_dictionary)),
            dictionary: dict,
            suggestion_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(10000).unwrap(),
            ))),
        })
    }

    pub fn check(&self, word: &str) -> bool {
        let lower_word = word.to_lowercase();
        if self.custom_dictionary.read().unwrap().contains(&lower_word) {
            return true;
        }
        if self.config.is_allowed_word(&lower_word) {
            return true;
        }
        if self.dictionary.check(word) {
            return true;
        }
        if self
            .dictionary
            .checker()
            .check_lower_as_title(true)
            .check(word)
        {
            return true;
        }
        if self
            .dictionary
            .checker()
            .check_lower_as_upper(true)
            .check(word)
        {
            return true;
        }
        false
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        debug!("Checking Cache: {:?}", word);
        // First try to get from cache with write lock since get() needs to modify LRU order
        if let Some(suggestions) = self.suggestion_cache.write().unwrap().get_mut(word) {
            debug!("Cache hit for {:?}", word);
            return suggestions.clone();
        }

        // If not in cache, generate suggestions
        let mut suggestions = Vec::new();
        self.dictionary.suggest(word, &mut suggestions);
        suggestions.truncate(5);
        if !suggestions.is_empty() {
            let word_case = self.get_word_case(&word);
            // mutate suggestions in place to match case

            for suggestion in &mut suggestions {
                match word_case {
                    WordCase::AllCaps => {
                        suggestion.make_ascii_uppercase();
                    }
                    WordCase::AllLower => {
                        suggestion.make_ascii_lowercase();
                    }
                    WordCase::TitleCase => {
                        // Leave it alone if it's a title case
                    }
                    WordCase::Unknown => {}
                }
            }

            self.suggestion_cache
                .write()
                .unwrap()
                .put(word.to_string(), suggestions.clone());
        }
        suggestions
    }

    fn get_word_case(&self, word: &str) -> WordCase {
        if word.chars().all(char::is_uppercase) {
            return WordCase::AllCaps;
        }
        if word.chars().all(char::is_lowercase) {
            return WordCase::AllLower;
        }
        if word.chars().next().unwrap().is_uppercase() {
            return WordCase::TitleCase;
        }
        WordCase::Unknown
    }
}

#[cfg(test)]
mod dictionary_tests {
    use super::*;

    fn get_dict() -> CodeDictionary {
        let config = CodebookConfig::new_no_file();
        config.add_word("badword").unwrap();
        let dict = CodeDictionary::new(
            Arc::new(config),
            "./tests/en_index.aff",
            "./tests/en_index.dic",
        )
        .unwrap();
        dict
    }

    #[test]
    fn test_spell_checking_custom_word() {
        let processor = get_dict();
        assert!(processor.check("badword"));
    }

    #[test]
    fn test_suggest() {
        let dict = get_dict();
        let suggestions = dict.suggest("wrld");
        println!("{:?}", suggestions);
        assert!(suggestions.contains(&"world".to_string()));
    }

    #[test]
    fn test_ignore_case() {
        let dict = get_dict();
        let check = dict.check("alice");
        assert!(check);
        let suggestions = dict.suggest("alice");
        println!("{:?}", suggestions);
        assert!(suggestions.contains(&"alice".to_string()));
    }
}
