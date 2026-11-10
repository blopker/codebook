use crate::checker::WordCandidate;
use crate::queries::{LANGUAGE_SETTINGS, LanguageSetting, LanguageType, get_language_setting};
use crate::splitter;
use log::{debug, error};
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor, QueryPredicateArg};
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

/// Global parser cache protected by a mutex. Serializes all tree-sitter
/// operations (create, parse, destroy) to protect external scanners that
/// use global mutable C state (e.g. tree-sitter-vhdl's static TokenTree).
/// A cached `None` records a grammar that failed to load, so the failure is
/// logged once instead of retried (and re-logged) on every spell check.
static PARSER_CACHE: LazyLock<Mutex<HashMap<LanguageType, Option<Parser>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// One `(#has-ancestor? @capture "kind" ...)` or `(#not-has-ancestor? ...)`
/// rule, pre-parsed so the hot path compares numeric kind ids. The capture is
/// dropped unless some ancestor of the captured node has one of the kinds
/// (`has-ancestor?`), or when one does (`not-has-ancestor?`).
struct AncestorRule {
    /// Position among all of the query's rules, for per-parse caching.
    id: usize,
    capture_index: u32,
    /// True for `not-has-ancestor?`.
    negated: bool,
    /// Indexed by node kind id. A grammar can give several ids the same
    /// name (aliases), so every id with a listed name is marked.
    kinds: Vec<bool>,
}

/// Pre-compiled query for a language, with its capture names.
struct CompiledQuery {
    query: Query,
    capture_names: Vec<String>,
    /// Indexed by `pattern_index`. Empty inner vec = no filtering for that
    /// pattern, which is the common case across all .scm files.
    ancestor_rules: Vec<Vec<AncestorRule>>,
    rule_count: usize,
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
        let mut rule_count = 0;
        let ancestor_rules = (0..query.pattern_count())
            .map(|i| {
                let mut rules = parse_ancestor_rules(&query, &lang, i, setting.type_);
                for rule in &mut rules {
                    rule.id = rule_count;
                    rule_count += 1;
                }
                rules
            })
            .collect();
        map.insert(
            setting.type_,
            CompiledQuery {
                query,
                capture_names,
                ancestor_rules,
                rule_count,
            },
        );
    }
    map
});

impl AncestorRule {
    /// Whether the capture passes, given whether it has a marked ancestor.
    fn accepts(&self, has_marked_ancestor: bool) -> bool {
        has_marked_ancestor != self.negated
    }
}

/// Extract `(#has-ancestor? @cap "kind" ...)` and `(#not-has-ancestor? ...)`
/// rules from a pattern's general predicates. Other custom predicates pass
/// through unchanged.
/// Malformed predicates, including kind names the grammar doesn't have,
/// panic at startup so .scm authors get an immediate error rather than a
/// silent no-op at runtime.
fn parse_ancestor_rules(
    query: &Query,
    language: &tree_sitter::Language,
    pattern_index: usize,
    language_type: LanguageType,
) -> Vec<AncestorRule> {
    let mut rules = Vec::new();
    for pred in query.general_predicates(pattern_index) {
        let op = &*pred.operator;
        let negated = match op {
            "has-ancestor?" => false,
            "not-has-ancestor?" => true,
            _ => continue,
        };
        let mut args = pred.args.iter();
        let capture_index = match args.next() {
            Some(QueryPredicateArg::Capture(i)) => *i,
            _ => panic!(
                "{:?}: #{op} must take a capture as its first argument",
                language_type
            ),
        };
        let names: Vec<&str> = args
            .map(|a| match a {
                QueryPredicateArg::String(s) => &**s,
                QueryPredicateArg::Capture(_) => panic!(
                    "{:?}: #{op} takes string node kinds after the capture",
                    language_type
                ),
            })
            .collect();
        assert!(
            !names.is_empty(),
            "{:?}: #{op} needs at least one node kind",
            language_type
        );
        let mut kinds = vec![false; language.node_kind_count()];
        for name in names {
            let mut found = false;
            for id in 0..language.node_kind_count() as u16 {
                if language.node_kind_is_named(id) && language.node_kind_for_id(id) == Some(name) {
                    kinds[id as usize] = true;
                    found = true;
                }
            }
            assert!(
                found,
                "{:?}: #{op} names unknown node kind {name:?} (only named kinds \
                 can be ancestors; anonymous tokens like \"(\" are always leaves)",
                language_type
            );
        }
        rules.push(AncestorRule {
            id: 0,
            capture_index,
            negated,
            kinds,
        });
    }
    rules
}

/// A node whose kind an ancestor rule marks, with no marked
/// ancestor of its own.
struct OutermostMarked {
    start_byte: usize,
    end_byte: usize,
    id: usize,
}

/// The outermost marked nodes under `root`, in document order and without
/// overlaps. One cursor pass over the tree: `Node::parent` searches down from
/// the root on every call, so walking up from each capture would cost
/// depth² per capture.
fn collect_outermost_marked(root: Node, kinds: &[bool]) -> Vec<OutermostMarked> {
    let mut found = Vec::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        let marked = kinds.get(node.kind_id() as usize) == Some(&true);
        // A zero-width node can't contain a capture, and captures that are
        // empty are dropped before rules apply.
        if marked && node.start_byte() < node.end_byte() {
            found.push(OutermostMarked {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                id: node.id(),
            });
        }
        if !marked && cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return found;
            }
        }
    }
}

/// Returns true if any ancestor of `node` has a kind marked in `kinds`, given
/// that rule's outermost marked nodes.
fn has_marked_ancestor(node: Node, outermost: &[OutermostMarked], kinds: &[bool]) -> bool {
    let (start, end) = (node.start_byte(), node.end_byte());
    let i = outermost.partition_point(|m| m.start_byte <= start);
    let Some(m) = i.checked_sub(1).map(|i| &outermost[i]) else {
        return false;
    };
    if end > m.end_byte || m.id == node.id() {
        return false;
    }
    if (m.start_byte, m.end_byte) == (start, end) {
        // Equal spans: `node` may be the marked node's ancestor rather than
        // its descendant, which byte ranges can't tell apart.
        return has_ancestor_kind(node, kinds);
    }
    true
}

/// Returns true if any ancestor of `node` has a kind marked in `kinds`.
fn has_ancestor_kind(node: Node, kinds: &[bool]) -> bool {
    let mut cur = node.parent();
    while let Some(parent) = cur {
        if kinds.get(parent.kind_id() as usize) == Some(&true) {
            return true;
        }
        cur = parent.parent();
    }
    false
}

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

/// `skip_ranges` must be sorted and non-overlapping (see
/// `merge_overlapping_ranges`), so only the last range starting at or before
/// `start` can contain the span.
fn is_within_skip_range(start: usize, end: usize, skip_ranges: &[SkipRange]) -> bool {
    let i = skip_ranges.partition_point(|r| r.start_byte <= start);
    i > 0 && end <= skip_ranges[i - 1].end_byte
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
pub fn extract_all_words<'a>(
    document_text: &'a str,
    language: LanguageType,
    tag_filter: &dyn Fn(&str) -> bool,
    skip_patterns: &[Regex],
) -> (Vec<WordCandidate<'a>>, HashSet<LanguageType>) {
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
struct ExtractionResult<'a> {
    candidates: Vec<WordCandidate<'a>>,
    languages: HashSet<LanguageType>,
}

/// Create a parser for a language's grammar, or None when the grammar can't
/// be loaded (e.g. a tree-sitter ABI version mismatch).
fn new_parser(setting: &LanguageSetting) -> Option<Parser> {
    let lang = setting.language()?;
    let mut parser = Parser::new();
    match parser.set_language(&lang) {
        Ok(()) => Some(parser),
        Err(e) => {
            error!(
                "Failed to load tree-sitter grammar for {:?}: {e}",
                setting.type_
            );
            None
        }
    }
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
fn extract_recursive<'a>(
    document_text: &'a str,
    start_byte: usize,
    end_byte: usize,
    language: LanguageType,
    tag_filter: &dyn Fn(&str) -> bool,
    skip_ranges: &[SkipRange],
    result: &mut ExtractionResult<'a>,
) {
    let language_setting = match get_language_setting(language) {
        Some(s) => s,
        None => {
            // No grammar (e.g. Text): word-split the whole range
            let text = &document_text[start_byte..end_byte];
            extract_words_from_text(text, start_byte, skip_ranges, &mut result.candidates);
            return;
        }
    };

    let region_text = &document_text[start_byte..end_byte];

    // Parse under global lock. This block must not panic: a panic while
    // holding PARSER_CACHE poisons the mutex and fails every spell check
    // for the rest of the process.
    let tree = {
        let mut cache = PARSER_CACHE.lock().unwrap();
        let parser = match cache.entry(language) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(new_parser(language_setting)),
        };
        parser
            .as_mut()
            .and_then(|parser| parser.parse(region_text, None))
    };

    let Some(tree) = tree else {
        // Grammar unavailable or parse failed: degrade to word-splitting the
        // region as plain text so it is still checked. debug-level because
        // this runs per check; the grammar failure itself was already logged
        // once by new_parser.
        debug!("Parsing {language:?} unavailable; checking region as plain text");
        extract_words_from_text(region_text, start_byte, skip_ranges, &mut result.candidates);
        return;
    };

    let root_node = tree.root_node();
    let compiled = COMPILED_QUERIES
        .get(&language)
        .expect("Language has a LanguageSetting but no compiled query; this should not happen");
    let mut cursor = QueryCursor::new();
    let provider = region_text.as_bytes();
    let mut matches_query = cursor.matches(&compiled.query, root_node, provider);

    // `@skip` ranges suppress any word extracted inside them, including
    // words from languages injected there. Injected ranges belong to the
    // injected language: this language's own captures over the same text
    // are dropped, so a broad capture (YAML's `(block_scalar)`) can coexist
    // with an injection of the same node. Both are applied after the loop
    // because matches arrive in document order, so a broad capture can land
    // before the match that covers its text.
    let first_candidate = result.candidates.len();
    let mut skipped_nodes: Vec<SkipRange> = Vec::new();
    let mut injected = Injected::default();

    // Each ancestor rule's outermost marked nodes in this tree,
    // collected the first time the rule is needed.
    let mut outermost_by_rule: Vec<Option<Vec<OutermostMarked>>> =
        (0..compiled.rule_count).map(|_| None).collect();

    while let Some(match_) = matches_query.next() {
        // Per-pattern ancestor rules. Cheap: the inner vec is empty for
        // every pattern in every .scm except the rare ones that declare a
        // rule, so we pay one `is_empty` check on the hot path.
        let ancestor_rules = &compiled.ancestor_rules[match_.pattern_index];

        // First pass: look for dynamic injection pairs in this match
        let mut injection_content: Option<tree_sitter::Node> = None;
        let mut injection_language_text: Option<&str> = None;

        for capture in match_.captures() {
            let tag = &compiled.capture_names[capture.index as usize];
            if tag == "injection.content" {
                injection_content = Some(capture.node);
            } else if tag == "injection.language" {
                injection_language_text = Some(capture.node.utf8_text(provider).unwrap_or(""));
            }
        }

        // Handle dynamic injection pair
        if let Some(content_node) = injection_content {
            if let Some(lang_text) = injection_language_text {
                let lowered = lang_text.trim().to_lowercase();
                let child_lang = LanguageType::from_str(&lowered);
                if let Ok(child_lang) = child_lang
                    && child_lang != LanguageType::Text
                {
                    let child_start = content_node.start_byte() + start_byte;
                    let child_end = content_node.end_byte() + start_byte;
                    if child_start < child_end {
                        result.languages.insert(child_lang);
                        let before = result.candidates.len();
                        extract_recursive(
                            document_text,
                            child_start,
                            child_end,
                            child_lang,
                            tag_filter,
                            skip_ranges,
                            result,
                        );
                        injected.record(child_start, child_end, before, result.candidates.len());
                    }
                }
            }
            continue;
        }

        // Second pass: handle text captures and static injections
        for capture in match_.captures() {
            let tag = &compiled.capture_names[capture.index as usize];
            let node = capture.node;
            let node_start = node.start_byte() + start_byte;
            let node_end = node.end_byte() + start_byte;

            if node_start >= node_end {
                continue;
            }

            // `_`-prefixed captures only feed predicates (`#eq? @_fn "require"`).
            if tag == "language" || tag == "injection.language" || tag.starts_with('_') {
                continue;
            }

            // Apply any ancestor rules that target this capture.
            if !ancestor_rules.is_empty()
                && ancestor_rules.iter().any(|rule| {
                    if rule.capture_index != capture.index {
                        return false;
                    }
                    let outermost = outermost_by_rule[rule.id]
                        .get_or_insert_with(|| collect_outermost_marked(root_node, &rule.kinds));
                    !rule.accepts(has_marked_ancestor(node, outermost, &rule.kinds))
                })
            {
                continue;
            }

            if tag == "skip" {
                skipped_nodes.push(SkipRange {
                    start_byte: node_start,
                    end_byte: node_end,
                });
                continue;
            }

            if let Some(lang_name) = tag.strip_prefix("injection.") {
                // Static injection: @injection.html, @injection.javascript, etc.
                if let Ok(child_lang) = LanguageType::from_str(lang_name)
                    && child_lang != LanguageType::Text
                {
                    result.languages.insert(child_lang);
                    let before = result.candidates.len();
                    extract_recursive(
                        document_text,
                        node_start,
                        node_end,
                        child_lang,
                        tag_filter,
                        skip_ranges,
                        result,
                    );
                    injected.record(node_start, node_end, before, result.candidates.len());
                }
                continue;
            }

            // Normal text capture: extract words if tag passes filter
            if !tag_filter(tag) {
                continue;
            }

            extract_words_from_node(
                node,
                region_text,
                start_byte,
                skip_ranges,
                &mut result.candidates,
            );
        }
    }

    if skipped_nodes.is_empty() && injected.regions.is_empty() {
        return;
    }
    skipped_nodes.sort_by_key(|r| r.start_byte);
    let skipped_nodes = merge_overlapping_ranges(skipped_nodes);
    injected.regions.sort_by_key(|r| r.start_byte);
    let injected_regions = merge_overlapping_ranges(injected.regions);
    // `retain` visits candidates in order, so the next injected candidate
    // span is always at `next_span`.
    let mut next_span = 0;
    let mut index = 0;
    result.candidates.retain(|c| {
        let i = index;
        index += 1;
        if i < first_candidate {
            return true;
        }
        if is_within_skip_range(c.start_byte, c.end_byte, &skipped_nodes) {
            return false;
        }
        while injected
            .candidate_spans
            .get(next_span)
            .is_some_and(|s| s.end <= i)
        {
            next_span += 1;
        }
        let from_injection = injected
            .candidate_spans
            .get(next_span)
            .is_some_and(|s| s.contains(&i));
        from_injection || !is_within_skip_range(c.start_byte, c.end_byte, &injected_regions)
    });
}

/// The regions a parse injected into other languages, and which candidates
/// those languages produced.
#[derive(Default)]
struct Injected {
    regions: Vec<SkipRange>,
    /// Indices into the candidate list, in ascending order: each injection
    /// appends its candidates after the previous one's.
    candidate_spans: Vec<std::ops::Range<usize>>,
}

impl Injected {
    fn record(&mut self, start_byte: usize, end_byte: usize, first: usize, end: usize) {
        self.regions.push(SkipRange {
            start_byte,
            end_byte,
        });
        if first < end {
            self.candidate_spans.push(first..end);
        }
    }
}

/// Node kinds that grammars use for escapes (`\n`, `\t`) nested in string
/// content: Swift calls them `str_escaped_char` and Typst `escape`; the rest
/// use `escape_sequence`.
const ESCAPE_KINDS: &[&str] = &["escape_sequence", "str_escaped_char", "escape"];

/// Extract words from a captured node's text, skipping escapes anywhere in
/// its subtree (Lua nests them under `string_content`, not the string).
/// Checking the raw text would glue an escape's letter onto the next word
/// ("\nis" → "nis").
fn extract_words_from_node<'a>(
    node: tree_sitter::Node,
    region_text: &'a str,
    region_offset: usize,
    skip_ranges: &[SkipRange],
    candidates: &mut Vec<WordCandidate<'a>>,
) {
    let mut segment_start = node.start_byte();
    // Most captures are leaves; only walk when there's a subtree.
    if node.child_count() > 0 {
        let mut cursor = node.walk();
        let mut depth = 0usize;
        'walk: loop {
            let current = cursor.node();
            let is_escape = depth > 0 && ESCAPE_KINDS.contains(&current.kind());
            if is_escape && current.start_byte() >= segment_start {
                let segment = &region_text[segment_start..current.start_byte()];
                extract_words_from_text(
                    segment,
                    segment_start + region_offset,
                    skip_ranges,
                    candidates,
                );
                segment_start = current.end_byte();
            }
            if !is_escape && cursor.goto_first_child() {
                depth += 1;
                continue;
            }
            while depth > 0 {
                if cursor.goto_next_sibling() {
                    continue 'walk;
                }
                cursor.goto_parent();
                depth -= 1;
            }
            break;
        }
    }
    let segment = &region_text[segment_start..node.end_byte()];
    extract_words_from_text(
        segment,
        segment_start + region_offset,
        skip_ranges,
        candidates,
    );
}

// =============================================================================
// Word extraction from plain text
// =============================================================================

fn extract_words_from_text<'a>(
    text: &'a str,
    base_offset: usize,
    skip_ranges: &[SkipRange],
    candidates: &mut Vec<WordCandidate<'a>>,
) {
    let mut split_buf = Vec::new();
    for (token_offset, token) in text.split_word_bound_indices() {
        // UAX #29 rules WB13a/WB13b keep tokens like "1000\u{202F}kWh" together
        // because NBSP/NNBSP are ExtendNumLet. The splitter assumes whitespace-
        // free input, so split each segmenter token on Unicode whitespace here.
        // Every char with the Unicode White_Space property is a word boundary
        // for spell-checking, regardless of language.
        for (sub_offset, word) in split_on_whitespace_indices(token) {
            if !is_alphabetic(word) {
                continue;
            }
            if has_unsupported_script(word) {
                continue;
            }
            let global_offset = base_offset + token_offset + sub_offset;
            if is_within_skip_range(global_offset, global_offset + word.len(), skip_ranges) {
                continue;
            }
            splitter::split_into(word, &mut split_buf);
            for split_word in &split_buf {
                if is_numeric(split_word.word) {
                    continue;
                }
                let word_start = global_offset + split_word.start_byte;
                let word_end = word_start + split_word.word.len();
                if is_within_skip_range(word_start, word_end, skip_ranges) {
                    continue;
                }
                candidates.push(WordCandidate {
                    word: split_word.word,
                    start_byte: word_start,
                    end_byte: word_end,
                });
            }
        }
    }
}

/// Iterate maximal non-whitespace runs in `s` with their byte offset.
/// `str::split_whitespace` discards offsets, which we need for diagnostic spans.
fn split_on_whitespace_indices(s: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut iter = s.char_indices().peekable();
    std::iter::from_fn(move || {
        while iter.peek().is_some_and(|&(_, c)| c.is_whitespace()) {
            iter.next();
        }
        let start = iter.peek()?.0;
        let mut end = s.len();
        while let Some(&(i, c)) = iter.peek() {
            if c.is_whitespace() {
                end = i;
                break;
            }
            iter.next();
        }
        Some((start, &s[start..end]))
    })
}

fn is_numeric(s: &str) -> bool {
    s.chars().any(|c| c.is_numeric())
}

fn is_alphabetic(c: &str) -> bool {
    c.chars().any(|c| c.is_alphabetic())
}

/// Returns true if any character in the word belongs to a script we have no
/// dictionary for. Filtering whole tokens (rather than substrings) avoids
/// emitting partial fragments like the leading "M" of "München" — Latin
/// scripts with diacritics still flow through to the Hunspell dictionaries
/// that handle French, German, etc.
fn has_unsupported_script(word: &str) -> bool {
    // Fast path: ASCII bytes can't belong to any unsupported script, and
    // `str::is_ascii` is a SIMD byte-loop, much cheaper than per-char script
    // lookups across the Unicode range table.
    if word.is_ascii() {
        return false;
    }
    word.chars().any(|c| {
        matches!(
            c.script(),
            Script::Han | Script::Hiragana | Script::Katakana | Script::Hangul | Script::Thai
        )
    })
}

/// Get a UTF-8 word from a string given the start and end bytes in utf16.
/// The offsets come from client-supplied LSP ranges, so they can be
/// inverted or out of bounds; saturate instead of panicking on underflow.
pub fn get_word_from_string(start_utf16: usize, end_utf16: usize, text: &str) -> String {
    let utf16_slice: Vec<u16> = text
        .encode_utf16()
        .skip(start_utf16)
        .take(end_utf16.saturating_sub(start_utf16))
        .collect();
    String::from_utf16_lossy(&utf16_slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_within_skip_range() {
        let ranges = [
            SkipRange {
                start_byte: 0,
                end_byte: 5,
            },
            SkipRange {
                start_byte: 10,
                end_byte: 20,
            },
            SkipRange {
                start_byte: 30,
                end_byte: 31,
            },
        ];
        assert!(is_within_skip_range(0, 5, &ranges));
        assert!(is_within_skip_range(12, 18, &ranges));
        assert!(is_within_skip_range(10, 20, &ranges));
        assert!(!is_within_skip_range(18, 22, &ranges)); // runs past the end
        assert!(!is_within_skip_range(6, 9, &ranges)); // between ranges
        assert!(!is_within_skip_range(8, 12, &ranges)); // starts before
        assert!(!is_within_skip_range(40, 45, &ranges)); // after the last
        assert!(!is_within_skip_range(0, 1, &[]));
    }

    fn python_query(source: &str) -> (Query, tree_sitter::Language) {
        let lang = get_language_setting(LanguageType::Python)
            .and_then(|s| s.language())
            .unwrap();
        (Query::new(&lang, source).unwrap(), lang)
    }

    #[test]
    fn test_ancestor_rules_resolve_kind_ids() {
        let (query, lang) = python_query(
            r#"((string_content) @string
                 (#not-has-ancestor? @string "type" "call")
                 (#has-ancestor? @string "block"))"#,
        );
        let rules = parse_ancestor_rules(&query, &lang, 0, LanguageType::Python);
        assert_eq!(rules.len(), 2);
        assert!(rules[0].negated && !rules[1].negated);
        assert!(rules[1].kinds.iter().any(|&k| k));
        let marked: Vec<&str> = (0..rules[0].kinds.len() as u16)
            .filter(|&id| rules[0].kinds[id as usize])
            .filter_map(|id| lang.node_kind_for_id(id))
            .collect();
        assert!(!marked.is_empty());
        assert!(marked.iter().all(|k| *k == "type" || *k == "call"));
        assert!(marked.contains(&"type") && marked.contains(&"call"));
    }

    /// The range lookup must agree with walking the parents, for every node.
    fn assert_marked_lookup_matches_parent_walk(
        language: LanguageType,
        text: &str,
        kinds: &[&str],
    ) {
        let lang = get_language_setting(language)
            .and_then(|s| s.language())
            .unwrap();
        let mut marks = vec![false; lang.node_kind_count()];
        for id in 0..lang.node_kind_count() as u16 {
            if lang
                .node_kind_for_id(id)
                .is_some_and(|k| kinds.contains(&k))
            {
                marks[id as usize] = true;
            }
        }
        let mut parser = Parser::new();
        parser.set_language(&lang).unwrap();
        let tree = parser.parse(text, None).unwrap();
        let outermost = collect_outermost_marked(tree.root_node(), &marks);
        assert!(!outermost.is_empty());

        let mut cursor = tree.walk();
        let mut checked = 0;
        loop {
            let node = cursor.node();
            assert_eq!(
                has_marked_ancestor(node, &outermost, &marks),
                has_ancestor_kind(node, &marks),
                "{:?} at {:?}",
                node.kind(),
                node.byte_range()
            );
            checked += 1;
            if cursor.goto_first_child() {
                continue;
            }
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    assert!(checked > 50);
                    return;
                }
            }
        }
    }

    #[test]
    fn test_marked_ancestor_lookup_latex() {
        let text = r"\documentclass[11pt]{article}
\begin{document}
\section{Intro \textbf{bold $x^2$}}
Text with \cite[see p.~3]{key} and $\text{math \emph{deep}}$.
\begin{equation}\label{eq:a} E = mc^2 \end{equation}
\href{http://x.com}{link text}
\end{document}
";
        assert_marked_lookup_matches_parent_walk(
            LanguageType::Latex,
            text,
            &[
                "begin",
                "inline_formula",
                "citation",
                "label_definition",
                "displayed_equation",
                "text",
            ],
        );
    }

    #[test]
    fn test_marked_ancestor_lookup_python() {
        let text = "def f(a: \"List[int]\", b: dict[\"K\", \"V\"]) -> \"R\":\n    x: \"T\" = \"s\"\n    return f\"{a}\"\n";
        assert_marked_lookup_matches_parent_walk(LanguageType::Python, text, &["type", "string"]);
    }

    #[test]
    #[should_panic(expected = "unknown node kind \"not_a_kind\"")]
    fn test_ancestor_rules_reject_unknown_kinds() {
        let (query, lang) =
            python_query(r#"((string_content) @string (#not-has-ancestor? @string "not_a_kind"))"#);
        parse_ancestor_rules(&query, &lang, 0, LanguageType::Python);
    }

    #[test]
    fn test_escape_split_offsets_after_multibyte_text() {
        // Byte offsets around an escape stay correct when multi-byte
        // characters come before it.
        let text = "x = \"äöü\\nwrold\\tnext\"\n";
        let (words, _) = extract_all_words(text, LanguageType::Python, &|_| true, &[]);
        let found: Vec<(&str, &str)> = words
            .iter()
            .map(|w| (w.word, &text[w.start_byte..w.end_byte]))
            .collect();
        assert!(found.iter().all(|(word, slice)| word == slice), "{found:?}");
        let strings: Vec<&str> = found
            .iter()
            .map(|(w, _)| *w)
            .filter(|w| *w != "x")
            .collect();
        assert_eq!(strings, ["äöü", "wrold", "next"]);
    }

    #[test]
    fn test_extract_words_plain_text() {
        let text = "HelloWorld calc_wrld";
        let (words, langs) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
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
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
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
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
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
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(word_strings.contains(&"comment"));
        assert!(!word_strings.contains(&"string"));
        assert!(!word_strings.contains(&"value"));
    }

    #[test]
    fn test_extract_words_with_skip_patterns() {
        let text = "check https://example.com this";
        let url_pattern = Regex::new(r"https?://[^\s]+").unwrap();
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[url_pattern]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(word_strings.contains(&"check"));
        assert!(word_strings.contains(&"this"));
        assert!(!word_strings.contains(&"https"));
        assert!(!word_strings.contains(&"example"));
    }

    #[test]
    fn test_extract_words_nbsp_inside_token() {
        // U+00A0 NO-BREAK SPACE between letters. The Unicode word segmenter
        // (UAX #29 WB13a/WB13b) keeps "foo\u{00A0}bar" as a single token;
        // we must split on the NBSP so the splitter sees clean words and
        // diagnostic spans line up with the original text.
        let text = "x foo\u{00A0}bar y";
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(strings.contains(&"foo"), "got {strings:?}");
        assert!(strings.contains(&"bar"), "got {strings:?}");

        let foo = words.iter().find(|w| w.word == "foo").unwrap();
        let bar = words.iter().find(|w| w.word == "bar").unwrap();
        assert_eq!(&text[foo.start_byte..foo.end_byte], "foo");
        assert_eq!(&text[bar.start_byte..bar.end_byte], "bar");
    }

    #[test]
    fn test_extract_words_nnbsp_between_number_and_unit() {
        // Regression for the panic that killed the LSP on locales using
        // U+202F NARROW NO-BREAK SPACE between a number and its unit
        // (e.g. French "1000 kWh"). Must not panic and must yield the unit.
        let text = "use 1000\u{202F}kWh today";
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(strings.contains(&"k"), "got {strings:?}");
        assert!(strings.contains(&"Wh"), "got {strings:?}");
        assert!(strings.contains(&"today"), "got {strings:?}");

        let wh = words.iter().find(|w| w.word == "Wh").unwrap();
        assert_eq!(&text[wh.start_byte..wh.end_byte], "Wh");
    }

    #[test]
    fn test_extract_words_ideographic_space() {
        // U+3000 IDEOGRAPHIC SPACE — Unicode White_Space, must act as a
        // boundary like ASCII space.
        let text = "hello\u{3000}world";
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(strings.contains(&"hello"), "got {strings:?}");
        assert!(strings.contains(&"world"), "got {strings:?}");
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
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(word_strings.contains(&"functin"));
        assert!(word_strings.contains(&"Hello"));
    }

    #[test]
    fn test_markdown_unknown_language_skipped() {
        let text = "# Hello\n\n```unknownlang\nbadwwword\n```\n";
        let (words, _) = extract_all_words(text, LanguageType::Markdown, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(!word_strings.contains(&"badwwword"));
    }

    #[test]
    fn test_markdown_html_block_injection() {
        let text = "# Hello\n\n<div>\n  <p>A misspeled word</p>\n</div>\n\nMore text.\n";
        let (words, langs) = extract_all_words(text, LanguageType::Markdown, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
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

    #[test]
    fn test_has_unsupported_script() {
        // CJK / Hiragana / Katakana / Hangul / Thai → unsupported
        assert!(has_unsupported_script("简体中文"));
        assert!(has_unsupported_script("繁體中文"));
        assert!(has_unsupported_script("にほんご"));
        assert!(has_unsupported_script("カタカナ"));
        assert!(has_unsupported_script("한국어"));
        assert!(has_unsupported_script("ภาษาไทย"));

        // Latin (incl. diacritics) and Cyrillic / Greek → supported
        assert!(!has_unsupported_script("hello"));
        assert!(!has_unsupported_script("café"));
        assert!(!has_unsupported_script("München"));
        assert!(!has_unsupported_script("naïve"));
        assert!(!has_unsupported_script("Tiếng")); // Vietnamese (vi_vn dict)
        assert!(!has_unsupported_script("Привет")); // Russian
        assert!(!has_unsupported_script("Ωμέγα")); // Greek

        // Mixed token: any unsupported char taints the whole word
        assert!(has_unsupported_script("hello世界"));
    }

    #[test]
    fn test_extract_words_skips_unsupported_scripts() {
        let text = "// 简体中文\n// 繁體中文\n// にほんご\n// ภาษาไทย\n// 한국어\n// hello world\n";
        let (words, _) = extract_all_words(text, LanguageType::Rust, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();

        // None of the CJK/Thai/Hangul tokens should appear as candidates.
        for forbidden in [
            "简", "体", "中", "文", "繁", "體", "に", "ほ", "ん", "ご", "ภ", "า", "ษ", "ไ", "ท",
            "ย", "한", "국", "어",
        ] {
            assert!(
                !word_strings.iter().any(|w| w.contains(forbidden)),
                "expected no candidate containing {forbidden:?}, got {word_strings:?}"
            );
        }

        // Latin words still come through.
        assert!(word_strings.contains(&"hello"));
        assert!(word_strings.contains(&"world"));
    }

    #[test]
    fn test_extract_words_keeps_supported_non_ascii() {
        // Latin diacritics, Cyrillic, and Vietnamese should still be extracted
        // for downstream dictionary lookup.
        let text = "café München Tiếng Привет";
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(word_strings.contains(&"café"), "got {word_strings:?}");
        assert!(word_strings.contains(&"München"), "got {word_strings:?}");
        assert!(word_strings.contains(&"Tiếng"), "got {word_strings:?}");
        assert!(word_strings.contains(&"Привет"), "got {word_strings:?}");
    }

    #[test]
    fn test_extract_words_mixed_diacritic_token_kept_whole() {
        // The PR-#252 regex `[^\x00-\x7F]+` would split "München" into "M"
        // and "nchen" — both garbage. We extract the whole token so the
        // German Hunspell dict can resolve it.
        let text = "Die Stadt München liegt in Bayern.";
        let (words, _) = extract_all_words(text, LanguageType::Text, &|_| true, &[]);
        let word_strings: Vec<&str> = words.iter().map(|w| w.word).collect();
        assert!(word_strings.contains(&"München"), "got {word_strings:?}");
        assert!(!word_strings.contains(&"M"));
        assert!(!word_strings.contains(&"nchen"));
    }
}
