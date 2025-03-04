use crate::splitter;
use log::{debug, info};

use crate::queries::{LanguageType, get_language_setting};
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};
use unicode_segmentation::UnicodeSegmentation;

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
        let chars: Vec<&str> = line.graphemes(true).collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if c == ":" && i + 1 < chars.len() {
                // Create a substring starting at the current position
                let byte_offset = line.char_indices().nth(i).unwrap().0;
                let remaining = &line[byte_offset..];

                if let Some((url_start, url_end)) = splitter::find_url_end(remaining) {
                    // Toss the current word
                    current_word.clear();
                    debug!(
                        "Found url: {}, skipping: {}",
                        &remaining[url_start..url_end],
                        url_end
                    );

                    // Count characters in the URL
                    let url_chars_count = remaining[..url_end].chars().count() as u32;

                    // Skip to after the URL
                    current_char += url_chars_count;

                    // Move index to after the URL
                    i += remaining[..url_end].chars().count();
                    continue;
                }
            }

            let is_contraction = c == "\'"
                && i > 0
                && i < chars.len() - 1
                && is_alphabetic(&chars[i - 1])
                && is_alphabetic(&chars[i + 1]);

            if is_alphabetic(c) || is_contraction {
                if current_word.is_empty() {
                    word_start_char = current_char;
                }
                current_word += c;
            } else {
                add_word_fn(&mut current_word, &mut words, word_start_char, current_line);
            }

            current_char += 1;
            i += 1;
        }

        add_word_fn(&mut current_word, &mut words, word_start_char, current_line);
        current_line += 1;
        current_char = 0;
    }

    words
}

fn is_alphabetic(s: &str) -> bool {
    // Return false for empty strings (optional, depends on your requirements)
    if s.is_empty() {
        return false;
    }

    // Check if all characters are alphabetic
    s.chars().all(|c| c.is_alphabetic())
}

/// Get a UTF-8 word from a string given the start and end indices.
pub fn get_word_from_string(start: usize, end: usize, text: &str) -> String {
    text.graphemes(true).skip(start).take(end - start).collect()
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

    #[test]
    fn test_get_word_from_string() {
        // Test with ASCII characters
        let text = "Hello World";
        assert_eq!(get_word_from_string(0, 5, text), "Hello");
        assert_eq!(get_word_from_string(6, 11, text), "World");

        // Test with partial words
        assert_eq!(get_word_from_string(2, 5, text), "llo");

        // Test with Unicode characters
        let unicode_text = "こんにちは世界";
        assert_eq!(get_word_from_string(0, 5, unicode_text), "こんにちは");
        assert_eq!(get_word_from_string(5, 7, unicode_text), "世界");

        // Test with emoji (which can be multi-codepoint)
        let emoji_text = "Hello 👨‍👩‍👧‍👦 World";
        assert_eq!(get_word_from_string(6, 7, emoji_text), "👨‍👩‍👧‍👦");
    }
    #[test]
    fn test_unicode_character_handling() {
        crate::log::init_test_logging();
        let text = "©<div>badword</div>";
        let words = get_words_from_text(text);
        println!("{:?}", words);

        // Make sure "badword" is included and correctly positioned
        assert!(words.iter().any(|(word, _)| word == "badword"));

        // If "badword" is found, verify its position
        if let Some((_, (start_char, line))) = words.iter().find(|(word, _)| word == "badword") {
            // The correct position should be 6 (after ©<div>)
            assert_eq!(
                *start_char, 6,
                "Expected 'badword' to start at character position 6"
            );
            assert_eq!(*line, 0, "Expected 'badword' to be on line 0");
        } else {
            panic!("Word 'badword' not found in the text");
        }
    }

    #[test]
    fn test_spell_checking_with_unicode() {
        crate::log::init_test_logging();
        let text = "©<div>badword</div>";

        // Mock spell check function that flags "badword"
        let results = find_locations(text, LanguageType::Html, |word| word != "badword");

        println!("{:?}", results);

        // Ensure "badword" is flagged
        let badword_result = results.iter().find(|loc| loc.word == "badword");
        assert!(badword_result.is_some(), "Expected 'badword' to be flagged");

        // Check if the location is correct
        if let Some(location) = badword_result {
            assert_eq!(
                location.locations.len(),
                1,
                "Expected exactly one location for 'badword'"
            );
            let range = &location.locations[0];

            // The word should start after "©<div>" which is 6 characters
            assert_eq!(range.start_char, 6, "Wrong start position for 'badword'");

            // The word should end after "badword" which is 13 characters from the start
            assert_eq!(range.end_char, 13, "Wrong end position for 'badword'");
        }
    }
}
