use crate::splitter;
use codebook_config::CodebookConfig;
use log::info;
use lru::LruCache;

use crate::queries::{
    get_common_dictionary, get_language_name_from_filename, get_language_setting, LanguageSetting,
    LanguageType,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Ord, Eq, PartialOrd)]
pub struct TextRange {
    pub start_char: u32,
    pub end_char: u32,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug)]
pub struct CodeDictionary {
    custom_dictionary: Arc<RwLock<HashSet<String>>>,
    dictionary: spellbook::Dictionary,
    pub make_suggestions: bool,
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
        for word in get_common_dictionary() {
            custom_dictionary.insert(word.to_string());
        }
        Ok(CodeDictionary {
            config,
            custom_dictionary: Arc::new(RwLock::new(custom_dictionary)),
            dictionary: dict,
            make_suggestions: true,
            suggestion_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(10000).unwrap(),
            ))),
        })
    }

    pub fn check(&self, word: &str) -> bool {
        self.custom_dictionary
            .read()
            .unwrap()
            .contains(word.to_lowercase().as_str())
            || self.dictionary.check(word)
        // self.lookup_cache.read().unwrap().contains(word) || self.dictionary.check(word)
    }

    pub fn add_to_dictionary(&self, strings: &str) {
        for line in strings.lines() {
            self.custom_dictionary
                .write()
                .unwrap()
                .insert(line.to_string());
        }
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        if !self.make_suggestions {
            return Vec::new();
        }
        info!("Checking Cache: {:?}", word);
        // First try to get from cache with write lock since get() needs to modify LRU order
        if let Some(suggestions) = self.suggestion_cache.write().unwrap().get_mut(word) {
            info!("Cache hit for {:?}", word);
            return suggestions.clone();
        }

        // If not in cache, generate suggestions
        let mut suggestions = Vec::new();
        self.dictionary.suggest(word, &mut suggestions);
        suggestions.truncate(5);
        if !suggestions.is_empty() {
            self.suggestion_cache
                .write()
                .unwrap()
                .put(word.to_string(), suggestions.clone());
        }
        suggestions
    }

    pub fn spell_check(&self, text: &str, language: &str) -> Vec<SpellCheckResult> {
        let lang_type = LanguageType::from_str(language);
        return self.spell_check_enum(text, lang_type);
    }

    pub fn spell_check_file(&self, path: &str) -> Vec<SpellCheckResult> {
        let lang_type = get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path).unwrap();
        return self.spell_check_enum(&file_text, lang_type);
    }

    pub fn spell_check_file_memory(&self, path: &str, contents: &str) -> Vec<SpellCheckResult> {
        let lang_type = get_language_name_from_filename(path);
        return self.spell_check_enum(&contents, lang_type);
    }

    fn spell_check_enum(
        &self,
        text: &str,
        language_type: Option<LanguageType>,
    ) -> Vec<SpellCheckResult> {
        let language = match language_type {
            None => None,
            Some(lang) => get_language_setting(lang),
        };
        match language {
            None => {
                return self.spell_check_text(text);
            }
            Some(lang) => {
                // if let Some(dictionary) = lang.language_dictionary {
                //     self.add_to_dictionary(dictionary);
                // }
                return self.spell_check_code(text, lang);
            }
        }
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
                        let range = TextRange {
                            start_char: word_start_char,
                            end_char: current_char,
                            start_line: current_line,
                            end_line: current_line,
                        };

                        // Check if we already have this misspelled word
                        if let Some(existing_result) =
                            results.iter_mut().find(|r| r.word == current_word)
                        {
                            existing_result.locations.push(range);
                        } else {
                            let mut locations = Vec::new();
                            locations.push(range);
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

    fn get_words_from_text(&self, text: &str) -> Vec<(String, (u32, u32))> {
        // Return Vec of words and their start char and line
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut word_start_char: u32 = 0;
        let mut current_char: u32 = 0;
        let mut current_line: u32 = 0;

        for line in text.lines() {
            for (i, c) in line.chars().enumerate() {
                let is_contraction = c == '\''
                    && i > 0
                    && i < line.len() - 1
                    && line.chars().nth(i - 1).unwrap().is_alphabetic()
                    && line.chars().nth(i + 1).unwrap().is_alphabetic();
                if c.is_alphabetic() || is_contraction {
                    if current_word.is_empty() {
                        word_start_char = current_char;
                    }
                    current_word.push(c);
                } else {
                    if !current_word.is_empty() {
                        words.push((current_word.clone(), (word_start_char, current_line)));
                        current_word.clear();
                    }
                }
                current_char += 1;
            }
            if !current_word.is_empty() {
                words.push((current_word.clone(), (word_start_char, current_line)));
                current_word.clear();
            }
            current_line += 1;
            current_char = 0;
        }
        words
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
        let mut word_locations: HashMap<String, Vec<TextRange>> = HashMap::new();
        let mut matches_query = cursor.matches(&query, root_node, text.as_bytes());

        while let Some(match_) = matches_query.next() {
            for capture in match_.captures {
                let node = capture.node;
                let node_text = node.utf8_text(text.as_bytes()).unwrap();
                let node_start = node.start_position();
                let current_line = node_start.row as u32;
                let current_column = node_start.column as u32;
                let words = self.get_words_from_text(node_text);
                info!("Found Capture:: {node_text:?}");
                info!("Words:: {words:?}");
                info!("Column: {current_column}");
                info!("Line: {current_line}");
                for (word_text, (text_start_char, text_line)) in words {
                    let split = splitter::split_camel_case(&word_text);
                    info!("Checking: {:?}", split);
                    for split_word in split {
                        if !self.check(&split_word.word) {
                            let offset = if text_line == 0 { current_column } else { 0 };
                            let base_start_char = text_start_char + offset;
                            let location = TextRange {
                                start_char: base_start_char + split_word.start_char,
                                end_char: base_start_char
                                    + split_word.start_char
                                    + split_word.word.chars().count() as u32,
                                start_line: text_line + current_line,
                                end_line: text_line + current_line,
                            };
                            if let Some(existing_result) = word_locations.get_mut(&split_word.word)
                            {
                                #[cfg(debug_assertions)]
                                if existing_result.contains(&location) {
                                    panic!("Two of the same locations found. Make a better query.")
                                }
                                existing_result.push(location);
                            } else {
                                word_locations.insert(split_word.word.clone(), vec![location]);
                            }
                        }
                    }
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

    fn get_dict() -> CodeDictionary {
        let config = Arc::new(CodebookConfig::default());
        let dict =
            CodeDictionary::new(config, "./tests/en_index.aff", "./tests/en_index.dic").unwrap();
        for word in EXTRA_WORDS {
            dict.add_to_dictionary(word);
        }
        dict
    }

    #[test]
    fn test_spell_checking() {
        let processor = get_dict();

        let text = "HelloWorld calc_wrld";
        let misspelled = processor.spell_check_enum(text, None);
        println!("{:?}", misspelled);
        assert!(misspelled.iter().any(|r| r.word == "wrld"));
    }

    #[test]
    fn test_get_words_from_text() {
        let dict = get_dict();
        let text = r#"
            HelloWorld calc_wrld
            I'm a contraction, don't ignore me
            this is a 3rd line.
            "#;
        let expected = vec![
            ("HelloWorld", (12, 1)),
            ("calc", (23, 1)),
            ("wrld", (28, 1)),
            ("I'm", (12, 2)),
            ("a", (16, 2)),
            ("contraction", (18, 2)),
            ("don't", (31, 2)),
            ("ignore", (37, 2)),
            ("me", (44, 2)),
            ("this", (12, 3)),
            ("is", (17, 3)),
            ("a", (20, 3)),
            ("rd", (23, 3)),
            ("line", (26, 3)),
        ];
        let words = dict.get_words_from_text(text);
        println!("{:?}", words);
        for (i, w) in expected.into_iter().enumerate() {
            assert_eq!(words[i], (w.0.to_string(), w.1));
        }
    }
}
