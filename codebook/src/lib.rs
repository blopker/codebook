pub mod downloader;
mod queries;
mod splitter;
use lru::LruCache;

use crate::queries::{get_language_name_from_filename, get_language_setting, LanguageSetting};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Clone, PartialEq)]
pub struct SpellCheckResult {
    pub word: String,
    pub suggestions: Vec<String>,
    pub locations: Vec<TextRange>,
}

impl SpellCheckResult {
    pub fn new(word: String, suggestions: Vec<&str>, locations: Vec<TextRange>) -> Self {
        SpellCheckResult {
            word,
            suggestions: suggestions.iter().map(|s| s.to_string()).collect(),
            locations,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextRange {
    pub start_char: u32,
    pub end_char: u32,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug)]
pub struct CodeDictionary {
    custom_dictionary: HashSet<String>,
    dictionary: spellbook::Dictionary,
    dictionary_lookup_cache: Arc<RwLock<LruCache<String, Vec<String>>>>,
}

impl CodeDictionary {
    pub fn new(aff_path: &str, dic_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let aff = std::fs::read_to_string(aff_path)?;
        let dic = std::fs::read_to_string(dic_path)?;
        let dict = spellbook::Dictionary::new(&aff, &dic)
            .map_err(|e| format!("Dictionary parse error: {}", e))?;

        Ok(CodeDictionary {
            custom_dictionary: HashSet::new(),
            dictionary: dict,
            dictionary_lookup_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(10000).unwrap(),
            ))),
        })
    }

    pub fn check(&self, word: &str) -> bool {
        self.dictionary.check(word)
        // self.dictionary_lookup_cache.read().unwrap().contains(word) || self.dictionary.check(word)
    }

    pub fn add_to_dictionary(&mut self, word: String) {
        self.custom_dictionary.insert(word);
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        println!("Checking Cache: {:?}", word);
        // First try to get from cache with write lock since get() needs to modify LRU order
        if let Some(suggestions) = self.dictionary_lookup_cache.write().unwrap().get_mut(word) {
            println!("Cache hit for {:?}", word);
            return suggestions.clone();
        }

        // If not in cache, generate suggestions
        let mut suggestions = Vec::new();
        self.dictionary.suggest(word, &mut suggestions);
        if !suggestions.is_empty() {
            self.dictionary_lookup_cache
                .write()
                .unwrap()
                .put(word.to_string(), suggestions.clone());
        }
        suggestions
    }

    pub fn spell_check(&self, text: &str, language: &str) -> Vec<SpellCheckResult> {
        // print!("language: {:?}", language);
        let lang = get_language_setting(language);
        match lang {
            None => {
                return self.spell_check_text(text);
            }
            Some(lang) => {
                return self.spell_check_code(text, lang);
            }
        }
    }

    pub fn spell_check_file(&self, path: &str) -> Vec<SpellCheckResult> {
        let lang_name = get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path).unwrap();
        return self.spell_check(&file_text, &lang_name);
    }

    pub fn spell_check_file_memory(&self, path: &str, contents: &str) -> Vec<SpellCheckResult> {
        let lang_name = get_language_name_from_filename(path);
        return self.spell_check(&contents, &lang_name);
    }

    fn spell_check_text(&self, text: &str) -> Vec<SpellCheckResult> {
        let mut results: Vec<SpellCheckResult> = Vec::new();
        let mut current_word = String::new();
        let mut word_start_char = 0;
        let mut current_char = 0;
        let mut current_line = 0;

        // Process each character in the text
        for c in text.chars() {
            if c.is_alphabetic() {
                if current_word.is_empty() {
                    word_start_char = current_char;
                }
                current_word.push(c);
            } else {
                // Word boundary found
                if !current_word.is_empty() {
                    // Check if word is in dictionary
                    if !self.check(&current_word) {
                        // Word not found in dictionary
                        let mut locations = Vec::new();
                        locations.push(TextRange {
                            start_char: word_start_char,
                            end_char: current_char,
                            start_line: current_line,
                            end_line: current_line,
                        });

                        // Check if we already have this misspelled word
                        if let Some(existing_result) =
                            results.iter_mut().find(|r| r.word == current_word)
                        {
                            existing_result.locations.push(locations[0].clone());
                        } else {
                            results.push(SpellCheckResult {
                                word: current_word.clone(),
                                suggestions: self.suggest(&current_word),
                                locations,
                            });
                        }
                    }
                    current_word.clear();
                }

                if c == '\n' {
                    current_line += 1;
                    current_char = 0;
                    continue;
                }
            }
            current_char += 1;
        }

        // Check the last word if text doesn't end with punctuation
        if !current_word.is_empty() {
            if !self.check(current_word.as_str()) {
                let locations = vec![TextRange {
                    start_char: word_start_char,
                    end_char: current_char,
                    start_line: current_line,
                    end_line: current_line,
                }];
                results.push(SpellCheckResult {
                    word: current_word.clone(),
                    suggestions: self.suggest(&current_word),
                    locations,
                });
            }
        }

        results
    }

    fn spell_check_code(
        &self,
        text: &str,
        language_setting: &LanguageSetting,
    ) -> Vec<SpellCheckResult> {
        let mut parser = Parser::new();
        let language = language_setting.language().unwrap();
        parser.set_language(&language).unwrap();

        let tree = parser.parse(text, None).unwrap();
        let root_node = tree.root_node();

        let query = Query::new(&language, language_setting.query).unwrap();
        let mut cursor = QueryCursor::new();
        let mut word_locations = HashMap::new();
        let mut matches_query = cursor.matches(&query, root_node, text.as_bytes());

        while let Some(match_) = matches_query.next() {
            for capture in match_.captures {
                let node = capture.node;
                let node_text = node.utf8_text(text.as_bytes()).unwrap();

                let node_start = node.start_position();
                let node_text_bytes = node_text.as_bytes();

                // Split the node text into words with their positions
                let word_boundaries: Vec<(String, usize, usize)> = node_text
                    .split(|c: char| !c.is_alphanumeric())
                    .enumerate()
                    .filter(|(_, word)| !word.is_empty())
                    .flat_map(|(_, word)| {
                        // First split by whitespace and punctuation
                        word.split_whitespace().flat_map(|w| {
                            // Then split camel case
                            splitter::split_camel_case(w).into_iter().map(move |part| {
                                // Find the exact position of this part in the original text
                                let mut start = 0;
                                let mut found = false;
                                for (idx, _) in node_text.match_indices(&part) {
                                    // Verify this is a whole word by checking boundaries
                                    let before = if idx > 0 {
                                        node_text
                                            .chars()
                                            .nth(idx - 1)
                                            .map_or(true, |c| !c.is_alphanumeric())
                                    } else {
                                        true
                                    };
                                    let after = if idx + part.len() < node_text.len() {
                                        node_text
                                            .chars()
                                            .nth(idx + part.len())
                                            .map_or(true, |c| !c.is_alphanumeric())
                                    } else {
                                        true
                                    };
                                    if before && after {
                                        start = idx;
                                        found = true;
                                        break;
                                    }
                                }
                                if found {
                                    (part.to_string(), start, start + part.len())
                                } else {
                                    (part.to_string(), 0, 0) // This should rarely happen
                                }
                            })
                        })
                    })
                    .collect();

                for (word, word_start, word_end) in word_boundaries {
                    if self.custom_dictionary.contains(&word) || self.check(&word) {
                        continue;
                    }

                    // Count lines and columns up to the word
                    let mut lines = 0;
                    let mut last_newline_pos = 0;
                    for (idx, &byte) in node_text_bytes[..word_start].iter().enumerate() {
                        if byte == b'\n' {
                            lines += 1;
                            last_newline_pos = idx + 1;
                        }
                    }

                    // Calculate start and end positions
                    let start_col = word_start - last_newline_pos;
                    let end_col = word_end - last_newline_pos;
                    let start_line = node_start.row + lines;

                    let (final_start_col, final_end_col) = if lines == 0 {
                        (node_start.column + start_col, node_start.column + end_col)
                    } else {
                        (start_col, end_col)
                    };

                    word_locations
                        .entry(word.clone())
                        .or_insert_with(Vec::new)
                        .push(TextRange {
                            start_char: u32::try_from(final_start_col).unwrap(),
                            end_char: u32::try_from(final_end_col).unwrap(),
                            start_line: u32::try_from(start_line).unwrap(),
                            end_line: u32::try_from(start_line).unwrap(),
                        });
                }
            }
        }

        word_locations
            .keys()
            .into_iter()
            .map(|word| SpellCheckResult {
                word: word.clone(),
                suggestions: self.suggest(&word),
                locations: word_locations.get(word).cloned().unwrap_or_default(),
            })
            .collect()
    }
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    static EXTRA_WORDS: &'static [&'static str] = &["http", "https", "www", "viewport", "UTF"];

    fn get_processor() -> CodeDictionary {
        let mut cdict =
            CodeDictionary::new("./tests/en_index.aff", "./tests/en_index.dic").unwrap();
        for word in EXTRA_WORDS {
            cdict.add_to_dictionary(word.to_string());
        }
        cdict
    }

    #[test]
    fn test_spell_checking() {
        let processor = get_processor();

        let text = "HelloWorld calc_wrld";
        let misspelled = processor.spell_check(text, "text");
        println!("{:?}", misspelled);
        assert!(misspelled.iter().any(|r| r.word == "wrld"));
    }
}
