use crate::checker::WordCandidate;
use crate::queries::{LanguageType, get_language_setting};
use crate::regions::TextRegion;
use crate::splitter;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};
use unicode_segmentation::UnicodeSegmentation;

/// Global parser cache protected by a mutex. Serializes all tree-sitter
/// operations (create, parse, destroy) to protect external scanners that
/// use global mutable C state (e.g. tree-sitter-vhdl's static TokenTree).
static PARSER_CACHE: LazyLock<Mutex<HashMap<LanguageType, Parser>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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

/// Check if a word at [start, end) is entirely within any skip range
fn is_within_skip_range(start: usize, end: usize, skip_ranges: &[SkipRange]) -> bool {
    skip_ranges
        .iter()
        .any(|r| start >= r.start_byte && end <= r.end_byte)
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

// =============================================================================
// Stage 2: Node Extraction
// =============================================================================

/// A text span extracted from a tree-sitter query match or plain text region.
/// Coordinates are in original-document byte offsets.
#[derive(Debug, Clone)]
pub struct TextNode {
    /// Byte range start in the original document
    pub start_byte: usize,
    /// Byte range end in the original document
    pub end_byte: usize,
    /// The text content of this node
    pub text: String,
}

/// Extract spellcheckable text nodes from a region.
/// For code regions, uses tree-sitter parsing and queries.
/// For text/markdown prose regions, returns the whole region as one node.
/// All byte offsets are in original document coordinates.
pub fn extract_nodes(
    document_text: &str,
    region: &TextRegion,
    tag_filter: &dyn Fn(&str) -> bool,
) -> Vec<TextNode> {
    let region_text = &document_text[region.start_byte..region.end_byte];

    match region.language {
        LanguageType::Text | LanguageType::Markdown => {
            // Plain text / markdown prose: the whole region is one node
            vec![TextNode {
                start_byte: region.start_byte,
                end_byte: region.end_byte,
                text: region_text.to_string(),
            }]
        }
        _ => {
            // Code: parse with tree-sitter, run query, extract captured nodes
            extract_nodes_with_treesitter(
                region_text,
                region.start_byte,
                region.language,
                tag_filter,
            )
        }
    }
}

/// Parse text with tree-sitter and extract nodes matching the language's query.
fn extract_nodes_with_treesitter(
    text: &str,
    base_offset: usize,
    language: LanguageType,
    tag_filter: &dyn Fn(&str) -> bool,
) -> Vec<TextNode> {
    let language_setting = match get_language_setting(language) {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Parse under global lock to protect external scanners with global C state.
    let tree = {
        let mut cache = PARSER_CACHE.lock().unwrap();
        let parser = cache.entry(language).or_insert_with(|| {
            let mut parser = Parser::new();
            let lang = language_setting.language().unwrap();
            parser.set_language(&lang).unwrap();
            parser
        });
        parser.parse(text, None).unwrap()
    };

    let root_node = tree.root_node();
    let lang = language_setting.language().unwrap();
    let query = Query::new(&lang, language_setting.query).unwrap();
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let provider = text.as_bytes();
    let mut matches_query = cursor.matches(&query, root_node, provider);

    let mut nodes = Vec::new();
    while let Some(match_) = matches_query.next() {
        for capture in match_.captures {
            let tag = &capture_names[capture.index as usize];
            // Skip internal tags and filtered tags
            if *tag == "language" || !tag_filter(tag) {
                continue;
            }
            let node = capture.node;
            let node_text = node.utf8_text(provider).unwrap();
            nodes.push(TextNode {
                start_byte: node.start_byte() + base_offset,
                end_byte: node.end_byte() + base_offset,
                text: node_text.to_string(),
            });
        }
    }
    nodes
}

// =============================================================================
// Stage 3: Word Extraction
// =============================================================================

/// Extract candidate words from text nodes, applying skip patterns.
/// All byte offsets are in original document coordinates.
pub fn extract_words(
    document_text: &str,
    nodes: &[TextNode],
    skip_patterns: &[Regex],
) -> Vec<WordCandidate> {
    // Compute skip ranges once against the full document
    let skip_ranges = find_skip_ranges(document_text, skip_patterns);

    let mut candidates = Vec::new();
    for node in nodes {
        extract_words_from_text(&node.text, node.start_byte, &skip_ranges, &mut candidates);
    }
    candidates
}

/// Extract words from a text span, applying skip ranges and word splitting.
fn extract_words_from_text(
    text: &str,
    base_offset: usize,
    skip_ranges: &[SkipRange],
    candidates: &mut Vec<WordCandidate>,
) {
    for (offset, word) in text.split_word_bound_indices() {
        if !is_alphabetic(word) {
            continue;
        }
        let global_offset = base_offset + offset;
        if is_within_skip_range(global_offset, global_offset + word.len(), skip_ranges) {
            continue;
        }
        let split = splitter::split(word);
        for split_word in split {
            if is_numeric(split_word.word) {
                continue;
            }
            let word_start = global_offset + split_word.start_byte;
            let word_end = word_start + split_word.word.len();
            if is_within_skip_range(word_start, word_end, skip_ranges) {
                continue;
            }
            candidates.push(WordCandidate {
                word: split_word.word.to_string(),
                start_byte: word_start,
                end_byte: word_end,
            });
        }
    }
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
    use crate::regions::TextRegion;

    #[test]
    fn test_extract_words_basic() {
        let text = "HelloWorld calc_wrld";
        let nodes = vec![TextNode {
            start_byte: 0,
            end_byte: text.len(),
            text: text.to_string(),
        }];
        let words = extract_words(text, &nodes, &[]);
        let word_strs: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(word_strs.contains(&"Hello"));
        assert!(word_strs.contains(&"World"));
        assert!(word_strs.contains(&"calc"));
        assert!(word_strs.contains(&"wrld"));
        assert_eq!(words.len(), 4);
    }

    #[test]
    fn test_extract_words_contraction() {
        let text = "I'm a contraction, wouldn't you agree'?";
        let nodes = vec![TextNode {
            start_byte: 0,
            end_byte: text.len(),
            text: text.to_string(),
        }];
        let words = extract_words(text, &nodes, &[]);
        let word_strs: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        let expected = ["I'm", "a", "contraction", "wouldn't", "you", "agree"];
        for e in &expected {
            assert!(word_strs.contains(e), "Expected word '{e}' not found");
        }
    }

    #[test]
    fn test_extract_nodes_plain_text() {
        let text = "hello world";
        let region = TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language: LanguageType::Text,
        };
        let nodes = extract_nodes(text, &region, &|_| true);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].text, "hello world");
        assert_eq!(nodes[0].start_byte, 0);
    }

    #[test]
    fn test_extract_nodes_code() {
        let text = "// a comment\nfn main() {}";
        let region = TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language: LanguageType::Rust,
        };
        let nodes = extract_nodes(text, &region, &|_| true);
        // Should have at least the comment node
        assert!(!nodes.is_empty());
        let comment_node = nodes.iter().find(|n| n.text.contains("comment"));
        assert!(comment_node.is_some(), "Should find comment node");
    }

    #[test]
    fn test_extract_nodes_with_base_offset() {
        // Simulate a code block starting at byte 50 in a larger document
        let code = "// hello world";
        let padded = format!("{}{}", " ".repeat(50), code);
        let region = TextRegion {
            start_byte: 50,
            end_byte: 50 + code.len(),
            language: LanguageType::Rust,
        };
        let nodes = extract_nodes(&padded, &region, &|_| true);
        assert!(!nodes.is_empty());
        // All node offsets should be >= 50
        for node in &nodes {
            assert!(node.start_byte >= 50, "Node offset should include base offset");
        }
    }

    #[test]
    fn test_extract_nodes_tag_filter() {
        let text = "// comment\nlet x = \"string\";";
        let region = TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language: LanguageType::Rust,
        };
        // Only allow comment tags
        let nodes = extract_nodes(text, &region, &|tag| tag.starts_with("comment"));
        for node in &nodes {
            // Should only have comment content
            assert!(
                node.text.contains("comment"),
                "Expected only comment nodes, got: {:?}",
                node.text
            );
        }
    }

    #[test]
    fn test_extract_words_with_skip_patterns() {
        let text = "check https://example.com this";
        let url_pattern = Regex::new(r"https?://[^\s]+").unwrap();
        let nodes = vec![TextNode {
            start_byte: 0,
            end_byte: text.len(),
            text: text.to_string(),
        }];
        let words = extract_words(text, &nodes, &[url_pattern]);
        let word_strs: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(word_strs.contains(&"check"));
        assert!(word_strs.contains(&"this"));
        // URL components should be skipped
        assert!(!word_strs.contains(&"https"));
        assert!(!word_strs.contains(&"example"));
    }

    #[test]
    fn test_get_word_from_string() {
        let text = "Hello World";
        assert_eq!(get_word_from_string(0, 5, text), "Hello");
        assert_eq!(get_word_from_string(6, 11, text), "World");
        assert_eq!(get_word_from_string(2, 5, text), "llo");

        let unicode_text = "こんにちは世界";
        assert_eq!(get_word_from_string(0, 5, unicode_text), "こんにちは");
        assert_eq!(get_word_from_string(5, 7, unicode_text), "世界");

        let emoji_text = "Hello 👨‍👩‍👧‍👦 World";
        assert_eq!(get_word_from_string(6, 17, emoji_text), "👨‍👩‍👧‍👦");
    }

    #[test]
    fn test_unicode_character_handling() {
        crate::logging::init_test_logging();
        let text = "©<div>badword</div>";
        let nodes = vec![TextNode {
            start_byte: 0,
            end_byte: text.len(),
            text: text.to_string(),
        }];
        let words = extract_words(text, &nodes, &[]);
        let badword = words.iter().find(|w| w.word == "badword");
        assert!(badword.is_some(), "Expected 'badword' to be found");
        let bw = badword.unwrap();
        assert_eq!(bw.start_byte, 7, "Expected 'badword' to start at byte 7");
        assert_eq!(bw.end_byte, 14, "Expected 'badword' to end at byte 14");
    }

    #[test]
    fn test_duplicate_word_locations_code() {
        let text = "// wrld foo wrld";
        let region = TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language: LanguageType::Rust,
        };
        let nodes = extract_nodes(text, &region, &|_| true);
        let words = extract_words(text, &nodes, &[]);
        let wrld_words: Vec<_> = words.iter().filter(|w| w.word == "wrld").collect();
        assert_eq!(
            wrld_words.len(),
            2,
            "Expected two occurrences of 'wrld'"
        );
    }
}
