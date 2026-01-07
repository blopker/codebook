use crate::splitter::{self};

use crate::queries::{LanguageType, get_language_setting};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Hash)]
pub struct TextRange {
    /// Start position in utf-8 byte offset
    pub start_byte: usize,
    /// End position in utf-8 byte offset
    pub end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SkipRange {
    /// Start position in utf-8 byte offset
    start_byte: usize,
    /// End position in utf-8 byte offset
    end_byte: usize,
}

impl SkipRange {
    fn contains(&self, pos: usize) -> bool {
        pos >= self.start_byte && pos < self.end_byte
    }
}

/// Find skip ranges from pattern matches in text.
fn find_skip_ranges(text: &str, patterns: &[Regex]) -> Vec<SkipRange> {
    if patterns.is_empty() {
        return Vec::new();
    }

    let mut ranges = Vec::new();

    for pattern in patterns {
        for regex_match in pattern.find_iter(text) {
            ranges.push(SkipRange {
                start_byte: regex_match.start(),
                end_byte: regex_match.end(),
            });
        }
    }

    ranges.sort_by_key(|r| r.start_byte);
    merge_overlapping_ranges(ranges)
}

/// Merge overlapping or adjacent ranges
fn merge_overlapping_ranges(ranges: Vec<SkipRange>) -> Vec<SkipRange> {
    if ranges.is_empty() {
        return ranges;
    }

    let mut merged = Vec::new();
    let mut current = ranges[0];

    for range in ranges.into_iter().skip(1) {
        if range.start_byte <= current.end_byte {
            current.end_byte = current.end_byte.max(range.end_byte);
        } else {
            merged.push(current);
            current = range;
        }
    }
    merged.push(current);
    merged
}

/// Helper struct to handle text position tracking and word extraction
struct TextProcessor {
    text: String,
    skip_ranges: Vec<SkipRange>,
}

impl TextProcessor {
    fn new(text: &str, skip_patterns: &[Regex]) -> Self {
        let skip_ranges = find_skip_ranges(text, skip_patterns);
        Self {
            text: text.to_string(),
            skip_ranges,
        }
    }

    fn should_skip(&self, start_byte: usize, word_len: usize) -> bool {
        let word_end = start_byte + word_len;
        self.skip_ranges
            .iter()
            .any(|range| range.contains(start_byte) || range.contains(word_end))
    }

    fn process_words_with_check<F>(&self, mut check_function: F) -> Vec<WordLocation>
    where
        F: FnMut(&str) -> bool,
    {
        // First pass: collect all unique words with their positions
        let estimated_words = (self.text.len() as f64 / 6.0).ceil() as usize;
        let mut word_positions: HashMap<&str, Vec<TextRange>> =
            HashMap::with_capacity(estimated_words);

        for (offset, word) in self.text.split_word_bound_indices() {
            if is_alphabetic(word) && !self.should_skip(offset, word.len()) {
                self.collect_split_words(word, offset, &mut word_positions);
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
        offset: usize,
        word_positions: &mut HashMap<&'a str, Vec<TextRange>>,
    ) {
        if !word.is_empty() {
            let split = splitter::split(word);
            for split_word in split {
                if !is_numeric(split_word.word) {
                    let word_start_byte = offset + split_word.start_byte;
                    let location = TextRange {
                        start_byte: word_start_byte,
                        end_byte: word_start_byte + split_word.word.len(),
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
    line_skip_patterns: &[Regex],
) -> Vec<WordLocation> {
    match language {
        LanguageType::Text => {
            // For text files, combine all patterns for substring matching
            let all_patterns: Vec<Regex> = skip_patterns
                .iter()
                .chain(line_skip_patterns.iter())
                .cloned()
                .collect();
            let processor = TextProcessor::new(text, &all_patterns);
            processor.process_words_with_check(|word| check_function(word))
        }
        _ => find_locations_code(
            text,
            language,
            |word| check_function(word),
            skip_patterns,
            line_skip_patterns,
        ),
    }
}

fn find_locations_code(
    text: &str,
    language: LanguageType,
    check_function: impl Fn(&str) -> bool,
    skip_patterns: &[Regex],
    line_skip_patterns: &[Regex],
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
    let mut word_locations: HashMap<String, HashSet<TextRange>> = HashMap::new();
    let provider = text.as_bytes();
    let mut matches_query = cursor.matches(&query, root_node, provider);

    // Find skip ranges from user patterns matched against the full source text
    // This allows patterns like 'vim\.opt\.\w+' to match across the full expression
    let user_skip_ranges = find_skip_ranges(text, line_skip_patterns);

    while let Some(match_) = matches_query.next() {
        for capture in match_.captures {
            let node = capture.node;
            let node_start_byte = node.start_byte();
            let node_end_byte = node.end_byte();

            // Check if the node falls within a user-defined skip range
            if user_skip_ranges
                .iter()
                .any(|r| r.contains(node_start_byte) || r.contains(node_end_byte.saturating_sub(1)))
            {
                continue;
            }

            let node_text = node.utf8_text(provider).unwrap();
            // Create processor with default patterns for substring matching within the node
            let processor = TextProcessor::new(node_text, skip_patterns);
            let words = processor.extract_words();

            // check words and fix locations relative to whole document
            for word_pos in words {
                if !check_function(&word_pos.word) {
                    for range in word_pos.locations {
                        let location = TextRange {
                            start_byte: range.start_byte + node_start_byte,
                            end_byte: range.end_byte + node_start_byte,
                        };
                        if let Some(existing_result) = word_locations.get_mut(&word_pos.word) {
                            #[cfg(debug_assertions)]
                            {
                                let added = existing_result.insert(location);
                                if !added {
                                    let word = word_pos.word.clone();
                                    panic!(
                                        "Two of the same locations found. Make a better query. Word: {word}, Location: {location:?}"
                                    )
                                }
                            }
                        } else {
                            let mut set = HashSet::new();
                            set.insert(location);
                            word_locations.insert(word_pos.word.clone(), set);
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
            locations: word_locations
                .get(word)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
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
    String::from_utf16_lossy(&utf16_slice)
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_spell_checking() {
        let text = "HelloWorld calc_wrld";
        let results = find_locations(text, LanguageType::Text, |_| false, &[], &[]);
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
            ("Hello", (13, 18)),
            ("World", (18, 23)),
            ("calc", (24, 28)),
            ("wrld", (29, 33)),
            ("I'm", (46, 49)),
            ("a", (50, 51)),
            ("contraction", (52, 63)),
            ("don't", (65, 70)),
            ("ignore", (71, 77)),
            ("me", (78, 80)),
            ("this", (93, 97)),
            ("is", (98, 100)),
            ("a", (101, 102)),
            ("rd", (104, 106)),
            ("line", (107, 111)),
        ];
        let processor = TextProcessor::new(text, &[]);
        let words = processor.extract_words();
        println!("{words:?}");
        for word in words {
            let loc = word.locations.first().unwrap();
            let pos = (loc.start_byte, loc.end_byte);
            assert!(
                expected.contains(&(word.word.as_str(), pos)),
                "Expected word '{}' to be at position {:?}",
                word.word,
                pos
            );
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
            let start_byte = pos.locations.first().unwrap().start_byte;
            let end_byte = pos.locations.first().unwrap().end_byte;
            assert_eq!(
                start_byte, 7,
                "Expected 'badword' to start at character position 7"
            );
            assert_eq!(end_byte, 14, "Expected 'badword' to be on end_byte 14");
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
