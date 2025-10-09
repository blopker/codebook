use crate::splitter::{self};

use crate::queries::{LanguageType, get_language_setting};
use log::info;
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
    fn contains(&self, pos: usize) -> bool {
        pos >= self.start_char && pos < self.end_char
    }
}

/// Helper struct to handle all text position tracking in one place
struct TextProcessor {
    text: String,
    line_starts: Vec<usize>, // Absolute character positions where each line starts
    skip_ranges: Vec<SkipRange>,
}

impl TextProcessor {
    fn new(text: &str, skip_patterns: &[Regex]) -> Self {
        let text = text.to_string();
        let skip_ranges = Self::find_skip_ranges(&text, skip_patterns);
        let line_starts = Self::calculate_line_starts(&text);

        Self {
            text,
            line_starts,
            skip_ranges,
        }
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
        Self::merge_overlapping_ranges(ranges)
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

    fn calculate_line_starts(text: &str) -> Vec<usize> {
        let mut line_starts = vec![0];
        let mut pos = 0;

        for line in text.lines() {
            pos += line.graphemes(true).count() + 1; // +1 for newline
            line_starts.push(pos);
        }

        line_starts
    }

    fn should_skip(&self, absolute_pos: usize, word_len: usize) -> bool {
        let word_end = absolute_pos + word_len;
        self.skip_ranges.iter().any(|range| {
            range.contains(absolute_pos)
                || range.contains(word_end.saturating_sub(1))
                || (absolute_pos < range.start_char && word_end > range.end_char)
        })
    }

    fn process_words_with_check<F>(&self, mut check_function: F) -> Vec<WordLocation>
    where
        F: FnMut(&str) -> bool,
    {
        // First pass: collect all unique words with their positions
        let mut word_positions: HashMap<&str, Vec<TextRange>> = HashMap::new();

        for (line_number, line) in self.text.lines().enumerate() {
            let line_start_abs = self.line_starts[line_number];
            let mut column = 0;

            for word in line.split_word_bounds() {
                if is_alphabetic(word) {
                    let absolute_pos = line_start_abs + column;
                    let word_len = word.graphemes(true).count();

                    if !self.should_skip(absolute_pos, word_len) {
                        self.collect_split_words(word, column, line_number, &mut word_positions);
                    }
                }
                column += word.graphemes(true).count();
            }
        }

        // Second pass: batch check unique words and filter
        let mut result_locations: HashMap<String, Vec<TextRange>> = HashMap::new();
        for (word_text, positions) in word_positions {
            if !check_function(word_text) {
                result_locations.insert(word_text.to_string(), positions);
            }
        }

        result_locations
            .into_iter()
            .map(|(word, locations)| WordLocation::new(word, locations))
            .collect()
    }

    fn extract_words(&self) -> Vec<WordLocation> {
        // Reuse the word collection logic by collecting all words (check always returns false)
        self.process_words_with_check(|_| false)
    }

    fn collect_split_words<'a>(
        &self,
        word: &'a str,
        column: usize,
        line_number: usize,
        word_positions: &mut HashMap<&'a str, Vec<TextRange>>,
    ) {
        if !word.is_empty() {
            let split = splitter::split(word);
            for split_word in split {
                if !is_numeric(split_word.word) {
                    let word_start_char = column as u32 + split_word.start_char;
                    let location = TextRange {
                        start_char: word_start_char,
                        end_char: word_start_char + split_word.word.chars().count() as u32,
                        line: line_number as u32,
                    };
                    let word_text = split_word.word;
                    word_positions.entry(word_text).or_default().push(location);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordRef<'a> {
    pub word: &'a str,
    pub position: (u32, u32), // (start_char, line)
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
        LanguageType::Text => {
            let processor = TextProcessor::new(text, skip_patterns);
            processor.process_words_with_check(|word| check_function(word))
        }
        _ => find_locations_code(text, language, |word| check_function(word), skip_patterns),
    }
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
            // Create processor on just this part of the document
            let processor = TextProcessor::new(node_text, skip_patterns);
            let words = processor.extract_words();
            // debug!("Found Capture: {node_text:?}");
            // debug!("Words: {words:?}");
            // debug!("Column: {current_column}");
            // debug!("Line: {current_line}");

            // check words and fix locations relative to whole document
            for word_pos in words {
                if !check_function(&word_pos.word) {
                    for range in word_pos.locations {
                        let offset = if range.line == 0 { current_column } else { 0 };
                        let base_start_char = range.start_char + offset;
                        let location = TextRange {
                            start_char: base_start_char,
                            end_char: base_start_char + word_pos.word.chars().count() as u32,
                            line: range.line + current_line,
                        };
                        if let Some(existing_result) = word_locations.get_mut(&word_pos.word) {
                            #[cfg(debug_assertions)]
                            if existing_result.contains(&location) {
                                panic!("Two of the same locations found. Make a better query.")
                            }
                            existing_result.push(location);
                        } else {
                            word_locations.insert(word_pos.word.clone(), vec![location]);
                        }
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

fn is_numeric(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

fn is_alphabetic(c: &str) -> bool {
    c.chars().any(|c| c.is_alphabetic())
}

/// Get a UTF-8 word from a string given the start and end bytes in utf16.
pub fn get_word_from_string(start_utf16: usize, end_utf16: usize, text: &str) -> String {
    let utf16_slice: Vec<u16> = text
        .encode_utf16()
        .skip(start_utf16)
        .take(end_utf16 - start_utf16)
        .collect();
    info!("UTF-16 slice: {:?}", utf16_slice);
    String::from_utf16_lossy(&utf16_slice)
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_spell_checking() {
        let text = "HelloWorld calc_wrld";
        let results = find_locations(text, LanguageType::Text, |_| false, &[]);
        println!("{results:?}");
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
        let processor = TextProcessor::new(text, &[]);
        let words = processor.extract_words();
        println!("{words:?}");
        for word in words {
            let loc = word.locations.first().unwrap();
            let pos = (loc.start_char, loc.line);
            assert!(expected.contains(&(word.word.as_str(), pos)));
        }
    }

    #[test]
    fn test_contraction() {
        let text = "I'm a contraction, wouldn't you agree'?";
        let processor = TextProcessor::new(text, &[]);
        let words = processor.extract_words();
        println!("{words:?}");
        let expected = ["I'm", "a", "contraction", "wouldn't", "you", "agree"];
        for word in words {
            assert!(expected.contains(&word.word.as_str()));
        }
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
        assert_eq!(get_word_from_string(6, 17, emoji_text), "👨‍👩‍👧‍👦");
    }
    #[test]
    fn test_unicode_character_handling() {
        crate::logging::init_test_logging();
        let text = "©<div>badword</div>";
        let processor = TextProcessor::new(text, &[]);
        let words = processor.extract_words();
        println!("{words:?}");

        // Make sure "badword" is included and correctly positioned
        assert!(words.iter().any(|word| word.word == "badword"));

        // If "badword" is found, verify its position
        if let Some(pos) = words.iter().find(|word| word.word == "badword") {
            // The correct position should be 6 (after ©<div>)
            let start_char = pos.locations.first().unwrap().start_char;
            let line = pos.locations.first().unwrap().line;
            assert_eq!(
                start_char, 6,
                "Expected 'badword' to start at character position 6"
            );
            assert_eq!(line, 0, "Expected 'badword' to be on line 0");
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
