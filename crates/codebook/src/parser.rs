use crate::splitter;
use log::{debug, info};

use crate::queries::{get_language_setting, LanguageType};
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Clone, Copy, PartialEq, Ord, Eq, PartialOrd)]
pub struct TextRange {
    pub start_char: u32,
    pub end_char: u32,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordLocation {
    pub word: String,
    pub locations: Vec<TextRange>,
}

impl WordLocation {
    pub fn new(word: String, locations: Vec<TextRange>) -> Self {
        Self { word, locations }
    }
}

pub fn find_locations(
    text: &str,
    language: LanguageType,
    check_function: impl Fn(&str) -> bool,
) -> Vec<WordLocation> {
    match language {
        LanguageType::Text => {
            return find_locations_text(text, check_function);
        }
        _ => {
            return find_locations_code(text, language, check_function);
        }
    }
}

fn find_locations_text(text: &str, check_function: impl Fn(&str) -> bool) -> Vec<WordLocation> {
    let mut results: Vec<WordLocation> = Vec::new();
    let words = get_words_from_text(text);

    // Check the last word if text doesn't end with punctuation
    for (current_word, (word_start_char, current_line)) in words {
        if !check_function(&current_word) {
            let locations = vec![TextRange {
                start_char: word_start_char,
                end_char: word_start_char + current_word.chars().count() as u32,
                line: current_line,
            }];
            results.push(WordLocation {
                word: current_word.clone(),
                locations,
            });
        }
    }

    results
}

fn find_locations_code(
    text: &str,
    language: LanguageType,
    check_function: impl Fn(&str) -> bool,
) -> Vec<WordLocation> {
    let language_setting =
        get_language_setting(language).expect("This _should_ never happen. Famous last words.");
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
            let words = get_words_from_text(node_text);
            debug!("Found Capture: {node_text:?}");
            debug!("Words: {words:?}");
            debug!("Column: {current_column}");
            debug!("Line: {current_line}");
            for (word_text, (text_start_char, text_line)) in words {
                info!("Checking: {:?}", word_text);
                if !check_function(&word_text) {
                    let offset = if text_line == 0 { current_column } else { 0 };
                    let base_start_char = text_start_char + offset;
                    let location = TextRange {
                        start_char: base_start_char,
                        end_char: base_start_char + word_text.chars().count() as u32,
                        line: text_line + current_line,
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
        .map(|word| WordLocation {
            word: word.clone(),
            locations: word_locations.get(word).cloned().unwrap_or_default(),
        })
        .collect()
}

fn get_words_from_text(text: &str) -> Vec<(String, (u32, u32))> {
    const MIN_WORD_LENGTH: usize = 3;
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
            if current_word.len() < MIN_WORD_LENGTH {
                current_word.clear();
                return;
            }
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

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_spell_checking() {
        let text = "HelloWorld calc_wrld";
        let results = find_locations(&text, LanguageType::Text, |_| false);
        println!("{:?}", results);
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_get_words_from_text() {
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
            ("contraction", (18, 2)),
            ("don't", (31, 2)),
            ("ignore", (37, 2)),
            ("this", (12, 3)),
            ("line", (26, 3)),
        ];
        let words = get_words_from_text(text);
        println!("{:?}", words);
        for (i, w) in expected.into_iter().enumerate() {
            assert_eq!(words[i], (w.0.to_string(), w.1));
        }
    }

    #[test]
    fn test_is_url() {
        crate::log::init_test_logging();
        let text = "https://www.google.com";
        let words = get_words_from_text(text);
        println!("{:?}", words);
        assert_eq!(words.len(), 0);
    }

    #[test]
    fn test_is_url_in_context() {
        crate::log::init_test_logging();
        let text = "Usez: https://intmainreturn0.com/ts-visualizer/ badwrd";
        let words = get_words_from_text(text);
        println!("{:?}", words);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0].0, "Usez");
        assert_eq!(words[1].0, "badwrd");
        assert_eq!(words[1].1, (48, 0));
    }

    #[test]
    fn test_contraction() {
        let text = "I'm a contraction, wouldn't you agree?";
        let words = get_words_from_text(text);
        println!("{:?}", words);
        assert_eq!(words[0].0, "I'm");
        assert_eq!(words[1].0, "contraction");
        assert_eq!(words[2].0, "wouldn't");
        assert_eq!(words[3].0, "you");
        assert_eq!(words[4].0, "agree");
    }
}
