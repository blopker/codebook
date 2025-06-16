use crate::splitter::{self};

use crate::queries::{LanguageType, get_language_setting};
use regex::Regex;
use ropey::Rope;
use std::collections::HashMap;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor, TextProvider};
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
struct TextProcessor<'a> {
    text: &'a Rope,
    skip_ranges: Vec<SkipRange>,
}

impl<'a> TextProcessor<'a> {
    fn new(text: &'a Rope, skip_patterns: &[Regex]) -> Self {
        // let text = text.to_string();
        let skip_ranges = Self::find_skip_ranges(text, skip_patterns);

        Self { text, skip_ranges }
    }

    fn find_skip_ranges(text: &Rope, patterns: &[Regex]) -> Vec<SkipRange> {
        let mut ranges = Vec::new();
        let str_text = text.to_string();
        for pattern in patterns {
            for regex_match in pattern.find_iter(&str_text) {
                let start_char = text.byte_to_char(regex_match.start());
                let end_char = text.byte_to_char(regex_match.end());
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
        let text_str = self.text.to_string();
        for (i, word) in text_str.unicode_word_indices() {
            let word_len = word.graphemes(true).count();
            let idx = self.text.byte_to_char(i);
            if !self.should_skip(idx, word_len) {
                self.collect_split_words(word, idx, &mut word_positions);
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

    fn extract_words(&self) -> Vec<(String, (u32, u32))> {
        // Reuse the word collection logic by collecting all words (check always returns false)
        let word_locations = self.process_words_with_check(|_| false);

        // Convert WordLocation format to the expected tuple format
        let mut result = Vec::new();
        for word_location in word_locations {
            for location in word_location.locations {
                result.push((
                    word_location.word.clone(),
                    (location.start_char, location.line),
                ));
            }
        }
        result
    }

    fn collect_split_words(
        &self,
        word: &'a str,
        idx: usize,
        word_positions: &mut HashMap<&'a str, Vec<TextRange>>,
    ) {
        if !word.is_empty() {
            let split = splitter::split(word);
            for split_word in split {
                if !is_numeric(split_word.word) {
                    // let word_start_char = column as u32 + split_word.start_char;
                    let line = self.text.char_to_line(idx);
                    let word_start_char = idx - self.text.line_to_char(line);
                    let location = TextRange {
                        start_char: word_start_char as u32 + split_word.start_char,
                        end_char: (word_start_char + split_word.word.graphemes(true).count())
                            as u32,
                        line: line as u32,
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
    text: &Rope,
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

pub struct TextProviderRope<'a>(pub &'a Rope);

impl<'a> TextProvider<&'a [u8]> for &'a TextProviderRope<'a> {
    type I = ChunksBytes<'a>;
    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        ChunksBytes(self.0.byte_slice(node.byte_range()).chunks())
    }
}
pub struct ChunksBytes<'a>(ropey::iter::Chunks<'a>);

impl<'a> Iterator for ChunksBytes<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(str::as_bytes)
    }
}

fn find_locations_code(
    text: &Rope,
    language: LanguageType,
    check_function: impl Fn(&str) -> bool,
    skip_patterns: &[Regex],
) -> Vec<WordLocation> {
    let language_setting =
        get_language_setting(language).expect("This _should_ never happen. Famous last words.");
    let mut parser = Parser::new();
    let language = language_setting.language().unwrap();
    parser.set_language(&language).unwrap();
    println!("{text:?}");
    // let text_str = text.chunks().collect();
    // let text_bytes = text.bytes().collect();
    let tree = parser
        .parse_with_options(
            &mut |start_byte, _| text.byte_slice(start_byte..).chunks().next().unwrap_or(""),
            None,
            None,
        )
        .unwrap();
    let root_node = tree.root_node();

    let query = Query::new(&language, language_setting.query).unwrap();
    let mut cursor = QueryCursor::new();
    let provider = TextProviderRope(text);
    let mut word_locations: HashMap<String, Vec<TextRange>> = HashMap::new();
    let mut matches_query = cursor.matches(&query, root_node, &provider);

    while let Some(match_) = matches_query.next() {
        for capture in match_.captures {
            let node = capture.node;
            // let node_text = node.utf8_text(text_bytes).unwrap();
            let node_start_byte = node.start_byte();
            let node_end_byte = node.end_byte();
            let node_start = node.start_position();
            let current_line = node_start.row as u32;
            let current_column = node_start.column as u32;
            let rope = Rope::from(text.byte_slice(node_start_byte..node_end_byte));
            let processor = TextProcessor::new(&rope, skip_patterns);
            let words = processor.extract_words();
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

fn is_numeric(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_spell_checking() {
        let text = Rope::from("HelloWorld calc_wrld");
        let results = find_locations(&text, LanguageType::Text, |_| false, &[]);
        println!("{:?}", results);
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_get_words_from_text() {
        let text = Rope::from(
            r#"
            HelloWorld calc_wrld
            I'm a contraction, don't ignore me
            this is a 3rd line.
            "#,
        );
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
        let processor = TextProcessor::new(&text, &[]);
        let words = processor.extract_words();
        println!("{:?}", words);
        for word in words {
            println!("{:?}", word);
            assert!(expected.contains(&(word.0.as_str(), word.1)));
        }
    }

    #[test]
    fn test_contraction() {
        let text = Rope::from("I'm a contraction, wouldn't you agree'?");
        let processor = TextProcessor::new(&text, &[]);
        let words = processor.extract_words();
        println!("{:?}", words);
        let expected = ["I'm", "a", "contraction", "wouldn't", "you", "agree"];
        for word in words {
            assert!(expected.contains(&word.0.as_str()));
        }
    }

    #[test]
    fn test_unicode_character_handling() {
        crate::logging::init_test_logging();
        let text = Rope::from("©<div>badword</div>");
        let processor = TextProcessor::new(&text, &[]);
        let words = processor.extract_words();
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
