use log::debug;
use lru::LruCache;

use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub trait Dictionary: Send + Sync {
    fn check(&self, word: &str) -> bool;
    fn suggest(&self, word: &str) -> Vec<String>;
}

enum WordCase {
    AllCaps,
    AllLower,
    TitleCase,
    Unknown,
}

#[derive(Debug)]
pub struct HunspellDictionary {
    dictionary: spellbook::Dictionary,
    suggestion_cache: Arc<RwLock<LruCache<String, Vec<String>>>>,
}

impl HunspellDictionary {
    pub fn new(aff_path: &str, dic_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let aff = std::fs::read_to_string(aff_path)?;
        let dic = std::fs::read_to_string(dic_path)?;
        let dict = spellbook::Dictionary::new(&aff, &dic)
            .map_err(|e| format!("Dictionary parse error: {}", e))?;
        Ok(HunspellDictionary {
            dictionary: dict,
            suggestion_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(10000).unwrap(),
            ))),
        })
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

impl Dictionary for HunspellDictionary {
    fn check(&self, word: &str) -> bool {
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

    fn suggest(&self, word: &str) -> Vec<String> {
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
            let word_case = self.get_word_case(word);
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
}

#[derive(Debug)]
pub struct TextDictionary {
    word_list: String,
}

impl Dictionary for TextDictionary {
    fn check(&self, word: &str) -> bool {
        let lower = word.to_ascii_lowercase();
        let words = self
            .word_list
            .lines()
            .filter(|s| !s.is_empty() && !s.starts_with('#'));
        for w in words {
            if w == lower {
                return true;
            }
        }
        false
    }
    fn suggest(&self, _word: &str) -> Vec<String> {
        vec![]
    }
}

impl TextDictionary {
    pub fn new(word_list: &str) -> Self {
        Self {
            word_list: word_list.to_ascii_lowercase(),
        }
    }
    pub fn new_from_path(path: &PathBuf) -> Self {
        let word_list = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read dictionary file: {}", path.display()))
            .to_ascii_lowercase();
        Self { word_list }
    }
}

#[cfg(test)]
mod dictionary_tests {
    use super::*;
    fn get_dict() -> HunspellDictionary {
        HunspellDictionary::new("./tests/en_index.aff", "./tests/en_index.dic").unwrap()
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
