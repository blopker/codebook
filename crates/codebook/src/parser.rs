use crate::checker::WordCandidate;
use crate::queries::{LANGUAGE_SETTINGS, LanguageType, get_language_setting};
use crate::splitter;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};
use unicode_segmentation::UnicodeSegmentation;

/// Global parser cache protected by a mutex. Serializes all tree-sitter
/// operations (create, parse, destroy) to protect external scanners that
/// use global mutable C state (e.g. tree-sitter-vhdl's static TokenTree).
static PARSER_CACHE: LazyLock<Mutex<HashMap<LanguageType, Parser>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Pre-compiled query for a language, with its capture names.
struct CompiledQuery {
    query: Query,
    capture_names: Vec<String>,
}

/// All tree-sitter queries compiled eagerly at startup. Since queries come
/// from static `include_str!` data, they never change at runtime. Compiling
/// them once here means bad queries panic immediately rather than hiding
/// until a user opens that file type.
static COMPILED_QUERIES: LazyLock<HashMap<LanguageType, CompiledQuery>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for setting in LANGUAGE_SETTINGS {
        let Some(lang) = setting.language() else {
            continue;
        };
        if setting.query.is_empty() {
            continue;
        }
        let query = Query::new(&lang, setting.query)
            .unwrap_or_else(|e| panic!("Failed to compile query for {:?}: {e}", setting.type_));
        let capture_names = query
            .capture_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        map.insert(
            setting.type_,
            CompiledQuery {
                query,
                capture_names,
            },
        );
    }
    map
});

#[derive(Debug, Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Hash)]
pub struct TextRange {
    /// Start position in utf-8 byte offset
    pub start_byte: usize,
    /// End position in utf-8 byte offset
    pub end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SkipRange {
    start_byte: usize,
    end_byte: usize,
}

fn is_within_skip_range(start: usize, end: usize, skip_ranges: &[SkipRange]) -> bool {
    skip_ranges
        .iter()
        .any(|r| start >= r.start_byte && end <= r.end_byte)
}

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
// Main entry point: recursive word extraction with injection support
// =============================================================================

/// Extract all candidate words from a document, recursively following
/// `@injection.*` captures in .scm query files to handle multi-language files.
///
/// Returns the candidates and the set of all languages encountered (for
/// dictionary loading).
pub fn extract_all_words(
    document_text: &str,
    language: LanguageType,
    tag_filter: &dyn Fn(&str) -> bool,
    skip_patterns: &[Regex],
) -> (Vec<WordCandidate>, HashSet<LanguageType>) {
    let skip_ranges = find_skip_ranges(document_text, skip_patterns);
    let mut result = ExtractionResult {
        candidates: Vec::new(),
        languages: HashSet::from([language]),
    };

    extract_recursive(
        document_text,
        0,
        document_text.len(),
        language,
        tag_filter,
        &skip_ranges,
        &mut result,
    );

    (result.candidates, result.languages)
}

/// Accumulated output from recursive word extraction.
struct ExtractionResult {
    candidates: Vec<WordCandidate>,
    languages: HashSet<LanguageType>,
}

/// Recursively extract words from a byte range of the document.
///
/// For languages with a tree-sitter grammar and .scm query:
///   - Text captures (`@string`, `@comment`, `@identifier.*`) → word-split
///   - Static injections (`@injection.{lang}`) → recurse with that language
///   - Dynamic injections (`@injection.content` + `@injection.language`) → read
///     the language name from the sibling capture, then recurse
///
/// For LanguageType::Text (no grammar): word-split the entire range.
fn extract_recursive(
    document_text: &str,
    start_byte: usize,
    end_byte: usize,
    language: LanguageType,
    tag_filter: &dyn Fn(&str) -> bool,
    skip_ranges: &[SkipRange],
    result: &mut ExtractionResult,
) {
    let language_setting = match get_language_setting(language) {
        Some(s) => s,
        None => {
            // No grammar (e.g. Text) — word-split the whole range
            let text = &document_text[start_byte..end_byte];
            extract_words_from_text(text, start_byte, skip_ranges, &mut result.candidates);
            return;
        }
    };

    let region_text = &document_text[start_byte..end_byte];

    // Parse under global lock
    let tree = {
        let mut cache = PARSER_CACHE.lock().unwrap();
        let parser = cache.entry(language).or_insert_with(|| {
            let mut parser = Parser::new();
            let lang = language_setting.language().unwrap();
            parser.set_language(&lang).unwrap();
            parser
        });
        parser.parse(region_text, None).unwrap()
    };

    let root_node = tree.root_node();
    let compiled = COMPILED_QUERIES
        .get(&language)
        .expect("Language has a LanguageSetting but no compiled query — this should not happen");
    let mut cursor = QueryCursor::new();
    let provider = region_text.as_bytes();
    let mut matches_query = cursor.matches(&compiled.query, root_node, provider);

    while let Some(match_) = matches_query.next() {
        // First pass: look for dynamic injection pairs in this match
        let mut injection_content: Option<tree_sitter::Node> = None;
        let mut injection_language_text: Option<String> = None;

        for capture in match_.captures {
            let tag = &compiled.capture_names[capture.index as usize];
            if tag == "injection.content" {
                injection_content = Some(capture.node);
            } else if tag == "injection.language" {
                injection_language_text =
                    Some(capture.node.utf8_text(provider).unwrap_or("").to_string());
            }
        }

        // Handle dynamic injection pair
        if let Some(content_node) = injection_content {
            if let Some(lang_text) = &injection_language_text {
                let child_lang = LanguageType::from_str(&lang_text.trim().to_lowercase());
                if let Ok(child_lang) = child_lang
                    && child_lang != LanguageType::Text
                {
                    let child_start = content_node.start_byte() + start_byte;
                    let child_end = content_node.end_byte() + start_byte;
                    if child_start < child_end {
                        result.languages.insert(child_lang);
                        extract_recursive(
                            document_text,
                            child_start,
                            child_end,
                            child_lang,
                            tag_filter,
                            skip_ranges,
                            result,
                        );
                    }
                }
            }
            continue;
        }

        // Second pass: handle text captures and static injections
        for capture in match_.captures {
            let tag = &compiled.capture_names[capture.index as usize];
            let node = capture.node;
            let node_start = node.start_byte() + start_byte;
            let node_end = node.end_byte() + start_byte;

            if node_start >= node_end {
                continue;
            }

            if tag == "language" || tag == "injection.language" {
                continue;
            }

            if let Some(lang_name) = tag.strip_prefix("injection.") {
                // Static injection: @injection.html, @injection.javascript, etc.
                if let Ok(child_lang) = LanguageType::from_str(lang_name)
                    && child_lang != LanguageType::Text
                {
                    result.languages.insert(child_lang);
                    extract_recursive(
                        document_text,
                        node_start,
                        node_end,
                        child_lang,
                        tag_filter,
                        skip_ranges,
                        result,
                    );
                }
                continue;
            }

            // Normal text capture — extract words if tag passes filter
            if !tag_filter(tag) {
                continue;
            }

            let node_text = node.utf8_text(provider).unwrap();
            extract_words_from_text(node_text, node_start, skip_ranges, &mut result.candidates);
        }
    }
}

// =============================================================================
// Word extraction from plain text
// =============================================================================

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
mod tests {
    use super::*;

    #[test]
    fn test_extract_words_plain_text() {
        let text = "HelloWorld calc_wrld";
        let (words, langs) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(word_strings.contains(&"Hello"));
        assert!(word_strings.contains(&"World"));
        assert!(word_strings.contains(&"calc"));
        assert!(word_strings.contains(&"wrld"));
        assert_eq!(words.len(), 4);
        assert!(langs.contains(&LanguageType::Text));
    }

    #[test]
    fn test_extract_words_contraction() {
        let text = "I'm a contraction, wouldn't you agree'?";
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        let expected = ["I'm", "a", "contraction", "wouldn't", "you", "agree"];
        for e in &expected {
            assert!(word_strings.contains(e), "Expected word '{e}' not found");
        }
    }

    #[test]
    fn test_extract_words_code() {
        let text = "// a comment\nfn main() {}";
        let (words, langs) = extract_all_words(text, LanguageType::Rust, &|_| true, &[]);
        assert!(!words.is_empty());
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(
            word_strings.contains(&"comment"),
            "Should find 'comment' in Rust comment"
        );
        assert!(langs.contains(&LanguageType::Rust));
    }

    #[test]
    fn test_extract_words_tag_filter() {
        let text = "// comment\nlet x = \"string value\";";
        let (words, _) = extract_all_words(
            text,
            LanguageType::Rust,
            &|tag| tag.starts_with("comment"),
            &[],
        );
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(word_strings.contains(&"comment"));
        assert!(!word_strings.contains(&"string"));
        assert!(!word_strings.contains(&"value"));
    }

    #[test]
    fn test_extract_words_with_skip_patterns() {
        let text = "check https://example.com this";
        let url_pattern = Regex::new(r"https?://[^\s]+").unwrap();
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[url_pattern]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(word_strings.contains(&"check"));
        assert!(word_strings.contains(&"this"));
        assert!(!word_strings.contains(&"https"));
        assert!(!word_strings.contains(&"example"));
    }

    #[test]
    fn test_extract_words_code_duplicates() {
        let text = "// wrld foo wrld";
        let (words, _) = extract_all_words(text, LanguageType::Rust, &|_| true, &[]);
        let wrld_words: Vec<_> = words.iter().filter(|w| w.word == "wrld").collect();
        assert_eq!(wrld_words.len(), 2, "Expected two occurrences of 'wrld'");
    }

    #[test]
    fn test_markdown_injection_discovers_languages() {
        let text =
            "# Hello\n\nSome text.\n\n```python\ndef foo(): pass\n```\n\n```bash\necho hi\n```\n";
        let (_, langs) = extract_all_words(text, LanguageType::Markdown, &|_| true, &[]);
        assert!(langs.contains(&LanguageType::Markdown));
        assert!(langs.contains(&LanguageType::Python));
        assert!(langs.contains(&LanguageType::Bash));
    }

    #[test]
    fn test_markdown_injection_extracts_code_words() {
        let text = "# Hello\n\n```python\ndef some_functin(): pass\n```\n";
        let (words, _) = extract_all_words(text, LanguageType::Markdown, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(word_strings.contains(&"functin"));
        assert!(word_strings.contains(&"Hello"));
    }

    #[test]
    fn test_markdown_unknown_language_skipped() {
        let text = "# Hello\n\n```unknownlang\nbadwwword\n```\n";
        let (words, _) = extract_all_words(text, LanguageType::Markdown, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(!word_strings.contains(&"badwwword"));
    }

    #[test]
    fn test_markdown_html_block_injection() {
        let text = "# Hello\n\n<div>\n  <p>A misspeled word</p>\n</div>\n\nMore text.\n";
        let (words, langs) = extract_all_words(text, LanguageType::Markdown, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word.as_str()).collect();
        assert!(langs.contains(&LanguageType::HTML));
        assert!(word_strings.contains(&"misspeled"));
        assert!(!word_strings.contains(&"div"));
    }

    #[test]
    fn test_get_word_from_string() {
        let text = "Hello World";
        assert_eq!(get_word_from_string(0, 5, text), "Hello");
        assert_eq!(get_word_from_string(6, 11, text), "World");

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
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let bad_word = words.iter().find(|w| w.word == "badword");
        assert!(bad_word.is_some(), "Expected 'badword' to be found");
        let bw = bad_word.unwrap();
        assert_eq!(bw.start_byte, 7);
        assert_eq!(bw.end_byte, 14);
    }
}
