use crate::splitter;
use codebook_config::CodebookConfig;
use log::{debug, info};
use lru::LruCache;

use crate::queries::{
    get_language_name_from_filename, get_language_setting, LanguageSetting, LanguageType,
};
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
    sync::{Arc, RwLock},
};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

static COMMON_DICTIONARY: &str = include_str!("../../word_lists/combined.gen.txt");
fn get_common_dictionary() -> impl Iterator<Item = &'static str> {
    COMMON_DICTIONARY.lines().filter(|l| !l.contains('#'))
}

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
            make_suggestions: false,
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
        self.dictionary.check(word)
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
        let words = self.get_words_from_text(text);

        // Check the last word if text doesn't end with punctuation
        for (current_word, (word_start_char, current_line)) in words {
            if !self.check(&current_word) {
                let locations = vec![TextRange {
                    start_char: word_start_char,
                    end_char: word_start_char + current_word.chars().count() as u32,
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

    /// Return Vec of words and their start char and line
    /// Skips URLs
    fn get_words_from_text(&self, text: &str) -> Vec<(String, (u32, u32))> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut word_start_char: u32 = 0;
        let mut current_char: u32 = 0;
        let mut current_line: u32 = 0;

        let add_word_fn = |current_word: &mut String,
                           words: &mut Vec<(String, (u32, u32))>,
                           word_start_char: u32,
                           current_line: u32| {
            if !current_word.is_empty() {
                let split = splitter::split_camel_case(&current_word);
                for split_word in split {
                    words.push((
                        split_word.word.clone(),
                        (word_start_char + split_word.start_char, current_line),
                    ));
                }
                current_word.clear();
            }
        };

        for line in text.lines() {
            let mut chars_to_skip = 0;
            for (i, c) in line.chars().enumerate() {
                if chars_to_skip > 0 {
                    chars_to_skip -= 1;
                    continue;
                }
                if c == ':' {
                    if let Some((url_start, url_end)) = splitter::find_url_end(&line[i..]) {
                        // Toss the current word and skip the URL
                        current_word.clear();
                        debug!(
                            "Found url: {}, skipping: {}",
                            &line[url_start + i..url_end + i],
                            url_end
                        );
                        chars_to_skip = url_end;
                        current_char += url_end as u32 + 1;
                        continue;
                    }
                }
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
                    add_word_fn(&mut current_word, &mut words, word_start_char, current_line);
                }
                current_char += 1;
            }
            add_word_fn(&mut current_word, &mut words, word_start_char, current_line);
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
                    info!("Checking: {:?}", word_text);
                    if !self.check(&word_text) {
                        let offset = if text_line == 0 { current_column } else { 0 };
                        let base_start_char = text_start_char + offset;
                        let location = TextRange {
                            start_char: base_start_char,
                            end_char: base_start_char + word_text.chars().count() as u32,
                            start_line: text_line + current_line,
                            end_line: text_line + current_line,
                        };
                        if let Some(existing_result) = word_locations.get_mut(&word_text) {
                            #[cfg(debug_assertions)]
                            if existing_result.contains(&location) {
                                panic!("Two of the same locations found. Make a better query.")
                            }
                            existing_result.push(location);
                        } else {
                            word_locations.insert(word_text.clone(), vec![location]);
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
mod dictionary_tests {
    use super::*;

    fn get_dict() -> CodeDictionary {
        let mut config = CodebookConfig::new_no_file();
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
            ("Hello", (12, 1)),
            ("World", (17, 1)),
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

    #[test]
    fn test_is_url() {
        crate::log::init_test_logging();
        let dict = get_dict();
        let text = "https://www.google.com";
        let words = dict.get_words_from_text(text);
        println!("{:?}", words);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_is_url_in_context() {
        crate::log::init_test_logging();
        let dict = get_dict();
        let text = "Usez: https://intmainreturn0.com/ts-visualizer/ badwrd";
        let words = dict.get_words_from_text(text);
        println!("{:?}", words);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].0, "Usez");
        assert_eq!(words[1].0, "badwrd");
        assert_eq!(words[1].1, (48, 0));
    }

    #[test]
    fn test_contraction() {
        let dict = get_dict();
        let text = "I'm a contraction, wouldn't you agree?";
        let words = dict.get_words_from_text(text);
        println!("{:?}", words);
        assert_eq!(words[0].0, "I'm");
        assert_eq!(words[1].0, "a");
        assert_eq!(words[2].0, "contraction");
        assert_eq!(words[3].0, "wouldn't");
        assert_eq!(words[4].0, "you");
        assert_eq!(words[5].0, "agree");
    }

    #[test]
    fn test_contraction_text() {
        let dict = get_dict();
        let text = "I'm a contraction, wouldn't you agre?";
        let words = dict.spell_check_text(text);
        println!("{:?}", words);
        assert_eq!(words[0].word, "agre");
    }
}
