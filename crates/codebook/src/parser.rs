use crate::splitter::{self};

use crate::queries::{LanguageType, get_language_setting};
use regex::Regex;
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct SkipRange {
    start_char: usize, // Start position in grapheme clusters
    end_char: usize,   // End position in grapheme clusters
}

impl SkipRange {
    fn overlaps(&self, start: usize, end: usize) -> bool {
        start < self.end_char && end > self.start_char
    }
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
    skip_patterns: &[Regex],
) -> Vec<WordLocation> {
    match language {
        LanguageType::Text => find_locations_text(text, check_function, skip_patterns),
        _ => find_locations_code(text, language, check_function, skip_patterns),
    }
}

fn find_locations_text(
    text: &str,
    check_function: impl Fn(&str) -> bool,
    skip_patterns: &[Regex],
) -> Vec<WordLocation> {
    let mut results: Vec<WordLocation> = Vec::new();
    let words = get_words_from_text(text, skip_patterns);

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
    skip_patterns: &[Regex],
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
    let provider = text.as_bytes();
    let mut matches_query = cursor.matches(&query, root_node, provider);

    while let Some(match_) = matches_query.next() {
        for capture in match_.captures {
            let node = capture.node;
            let node_text = node.utf8_text(provider).unwrap();
            let node_start = node.start_position();
            let current_line = node_start.row as u32;
            let current_column = node_start.column as u32;
            let words = get_words_from_text(node_text, skip_patterns);
            // debug!("Found Capture: {node_text:?}");
            // debug!("Words: {words:?}");
            // debug!("Column: {current_column}");
            // debug!("Line: {current_line}");
            for (word_text, (text_start_char, text_line)) in words {
                // debug!("Checking: {:?}", word_text);
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
        .map(|word| WordLocation {
            word: word.clone(),
            locations: word_locations.get(word).cloned().unwrap_or_default(),
        })
        .collect()
}

fn find_skip_ranges(text: &str, patterns: &[Regex]) -> Vec<SkipRange> {
    let mut ranges = Vec::new();

    for pattern in patterns {
        for regex_match in pattern.find_iter(text) {
            // Convert byte positions to grapheme positions
            let text_before_match = &text[..regex_match.start()];
            let start_char = text_before_match.graphemes(true).count();
            let match_str = regex_match.as_str();
            let end_char = start_char + match_str.graphemes(true).count();

            ranges.push(SkipRange {
                start_char,
                end_char,
            });
        }
    }

    // Sort ranges by start position and merge overlapping ones
    ranges.sort_by_key(|r| r.start_char);
    merge_overlapping_ranges(ranges)
}

fn merge_overlapping_ranges(ranges: Vec<SkipRange>) -> Vec<SkipRange> {
    if ranges.is_empty() {
        return ranges;
    }

    let mut merged = Vec::new();
    let mut current = ranges[0];

    for range in ranges.into_iter().skip(1) {
        if range.start_char <= current.end_char {
            // Overlapping or adjacent ranges - merge them
            current.end_char = current.end_char.max(range.end_char);
        } else {
            merged.push(current);
            current = range;
        }
    }
    merged.push(current);
    merged
}

fn get_words_from_text(text: &str, skip_patterns: &[Regex]) -> Vec<(String, (u32, u32))> {
    let skip_ranges = find_skip_ranges(text, skip_patterns);
    get_words_from_text_with_ranges(text, &skip_ranges)
}

fn get_words_from_text_with_ranges(
    text: &str,
    skip_ranges: &[SkipRange],
) -> Vec<(String, (u32, u32))> {
    let mut result = Vec::new();

    let add_word_fn = |current_word: &str,
                       words: &mut Vec<(String, (u32, u32))>,
                       word_start_char: usize,
                       current_line: usize| {
        if !current_word.is_empty() {
            let split = splitter::split(current_word);
            for split_word in split {
                if is_numeric(&split_word.word) {
                    continue;
                }
                words.push((
                    split_word.word,
                    (
                        word_start_char as u32 + split_word.start_char,
                        current_line as u32,
                    ),
                ));
            }
        }
    };

    // Calculate absolute character positions for each line start
    let mut line_start_positions = vec![0];
    let mut absolute_pos = 0;
    for line in text.lines() {
        absolute_pos += line.graphemes(true).count() + 1; // +1 for newline
        line_start_positions.push(absolute_pos);
    }

    for (line_number, line) in text.lines().enumerate() {
        let mut column = 0;
        let line_start_char = line_start_positions[line_number];

        let words = line.split_word_bounds();
        for word in words {
            let current_pos = line_start_char + column;
            let word_end_pos = current_pos + word.graphemes(true).count();

            // Check if this word overlaps with any skip range
            let should_skip = skip_ranges
                .iter()
                .any(|range| range.overlaps(current_pos, word_end_pos));

            if is_alphabetic(word) && !should_skip {
                add_word_fn(word, &mut result, column, line_number);
            }
            column += word.graphemes(true).count();
        }
    }
    result
}

fn is_numeric(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

fn is_alphabetic(c: &str) -> bool {
    c.chars().any(|c| c.is_alphabetic())
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
        let results = find_locations(text, LanguageType::Text, |_| false, &[]);
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
        let words = get_words_from_text(text, &[]);
        println!("{:?}", words);
        for (i, w) in expected.into_iter().enumerate() {
            assert_eq!(words[i], (w.0.to_string(), w.1));
        }
    }

    #[test]
    fn test_contraction() {
        let text = "I'm a contraction, wouldn't you agree'?";
        let words = get_words_from_text(text, &[]);
        println!("{:?}", words);
        assert_eq!(words[0].0, "I'm");
        assert_eq!(words[1].0, "a");
        assert_eq!(words[2].0, "contraction");
        assert_eq!(words[3].0, "wouldn't");
        assert_eq!(words[4].0, "you");
        assert_eq!(words[5].0, "agree");
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
        crate::logging::init_test_logging();
        let text = "©<div>badword</div>";
        let words = get_words_from_text(text, &[]);
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

    // Something is up with the HTML tree-sitter package
    // #[test]
    // fn test_spell_checking_with_unicode() {
    //     crate::log::init_test_logging();
    //     let text = "©<div>badword</div>";

    //     // Mock spell check function that flags "badword"
    //     let results = find_locations(text, LanguageType::Html, |word| word != "badword");

    //     println!("{:?}", results);

    //     // Ensure "badword" is flagged
    //     let badword_result = results.iter().find(|loc| loc.word == "badword");
    //     assert!(badword_result.is_some(), "Expected 'badword' to be flagged");

    //     // Check if the location is correct
    //     if let Some(location) = badword_result {
    //         assert_eq!(
    //             location.locations.len(),
    //             1,
    //             "Expected exactly one location for 'badword'"
    //         );
    //         let range = &location.locations[0];

    //         // The word should start after "©<div>" which is 6 characters
    //         assert_eq!(range.start_char, 6, "Wrong start position for 'badword'");

    //         // The word should end after "badword" which is 13 characters from the start
    //         assert_eq!(range.end_char, 13, "Wrong end position for 'badword'");
    //     }
    // }
}
