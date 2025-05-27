use lru::LruCache;

use std::{
    collections::HashSet,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::queries::LanguageType;
use crate::parser::{WordLocation, find_locations_with_hashset, find_locations};
use regex::Regex;

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
    check_cache: Arc<RwLock<LruCache<String, bool>>>,
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
            check_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(50000).unwrap(),
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
        // Check cache first
        if let Some(result) = self.check_cache.write().unwrap().get_mut(word) {
            return *result;
        }

        // If not in cache, perform the check
        let result = self.dictionary.check(word) 
            || self.dictionary.checker().check_lower_as_title(true).check_lower_as_upper(true).check(word);
        
        // Cache the result
        self.check_cache.write().unwrap().put(word.to_string(), result);
        
        result
    }

    fn suggest(&self, word: &str) -> Vec<String> {
        // debug!("Checking Cache: {:?}", word);
        // First try to get from cache with write lock since get() needs to modify LRU order
        if let Some(suggestions) = self.suggestion_cache.write().unwrap().get_mut(word) {
            // debug!("Cache hit for {:?}", word);
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
    words: HashSet<String>,
}

impl Dictionary for TextDictionary {
    fn check(&self, word: &str) -> bool {
        let lower = word.to_ascii_lowercase();
        self.words.contains(&lower)
    }
    fn suggest(&self, _word: &str) -> Vec<String> {
        vec![]
    }
}

impl TextDictionary {
    pub fn new(word_list: &str) -> Self {
        let words = word_list
            .lines()
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|s| s.to_ascii_lowercase())
            .collect();
        Self { words }
    }
    pub fn new_from_path(path: &PathBuf) -> Self {
        let word_list = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read dictionary file: {}", path.display()));
        Self::new(&word_list)
    }
    
    /// Get a reference to the internal HashSet for batch operations
    pub fn word_set(&self) -> &HashSet<String> {
        &self.words
    }
}

/// Integration helper to use TextDictionary with optimized parser functions
pub fn find_locations_with_text_dictionary(
    text: &str,
    language: LanguageType,
    dictionary: &TextDictionary,
    skip_patterns: &[Regex],
) -> Vec<WordLocation> {
    find_locations_with_hashset(text, language, dictionary.word_set(), skip_patterns)
}

/// Integration helper to use any Dictionary trait with optimized batch processing
pub fn find_locations_with_dictionary_batch(
    text: &str,
    language: LanguageType,
    dictionary: &dyn Dictionary,
    skip_patterns: &[Regex],
) -> Vec<WordLocation> {
    // For non-HashSet dictionaries, we still get deduplication benefits
    find_locations(text, language, |word| dictionary.check(word), skip_patterns)
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
