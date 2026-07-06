use codebook::queries::LanguageType;

use super::utils::{assert_spelling_with, get_processor_with_tags};

/// Sample Rust code with misspellings in comments, strings, and identifiers.
const RUST_SAMPLE: &str = r#"
    // A commet with a typo
    fn calculat_age() {
        let nmber = "a strng value";
    }
"#;

#[test]
fn test_no_filters_returns_all() {
    // Should find typos in all three categories
    assert_spelling_with(
        &get_processor_with_tags(vec![], vec![]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["commet", "calculat", "nmber", "strng"],
        &[],
    );
}

#[test]
fn test_include_comments_only() {
    assert_spelling_with(
        &get_processor_with_tags(vec!["comment"], vec![]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["commet"],
        // Identifiers, variables, and strings are excluded.
        &["calculat", "nmber", "strng"],
    );
}

#[test]
fn test_include_strings_only() {
    assert_spelling_with(
        &get_processor_with_tags(vec!["string"], vec![]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["strng"],
        &["commet", "calculat"],
    );
}

#[test]
fn test_include_identifiers_only() {
    assert_spelling_with(
        &get_processor_with_tags(vec!["identifier"], vec![]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["calculat", "nmber"],
        &["commet", "strng"],
    );
}

#[test]
fn test_exclude_identifiers() {
    assert_spelling_with(
        &get_processor_with_tags(vec![], vec!["identifier"]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["commet", "strng"],
        &["calculat", "nmber"],
    );
}

#[test]
fn test_exclude_specific_subtag() {
    // Exclude only identifier.variable, keep identifier.function
    assert_spelling_with(
        &get_processor_with_tags(vec![], vec!["identifier.variable"]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["commet", "calculat", "strng"],
        &["nmber"],
    );
}

#[test]
fn test_include_and_exclude_combined() {
    // Include comments and strings, but exclude string specifically
    assert_spelling_with(
        &get_processor_with_tags(vec!["comment", "string"], vec!["string"]),
        LanguageType::Rust,
        RUST_SAMPLE,
        &["commet"],
        // strng excluded by exclude_tags; calculat not in include_tags.
        &["strng", "calculat"],
    );
}

// =============================================================================
// Tag filters through injected regions (markdown → code blocks)
// =============================================================================

/// Markdown with a Python code block containing typos in a comment,
/// a function name (identifier), and a string.
const MARKDOWN_WITH_PYTHON: &str = r#"# A heading

Some prose.

```python
# A commet in python
def calculat_age():
    x = "a strng value"
```
"#;

#[test]
fn test_injection_no_filters_returns_all() {
    assert_spelling_with(
        &get_processor_with_tags(vec![], vec![]),
        LanguageType::Markdown,
        MARKDOWN_WITH_PYTHON,
        &["commet", "calculat", "strng"],
        &[],
    );
}

#[test]
fn test_injection_include_comments_only() {
    // include_tags = ["comment"] should only check comments,
    // even inside injected code blocks
    assert_spelling_with(
        &get_processor_with_tags(vec!["comment"], vec![]),
        LanguageType::Markdown,
        MARKDOWN_WITH_PYTHON,
        &["commet"],
        &["calculat", "strng"],
    );
}

#[test]
fn test_injection_exclude_identifiers() {
    // exclude_tags = ["identifier"] should suppress identifiers
    // in both prose and injected code blocks
    assert_spelling_with(
        &get_processor_with_tags(vec![], vec!["identifier"]),
        LanguageType::Markdown,
        MARKDOWN_WITH_PYTHON,
        &["commet", "strng"],
        &["calculat"],
    );
}

#[test]
fn test_injection_include_strings_only() {
    // include_tags = ["string"] should check strings in both
    // markdown prose (which uses @string) and injected python
    assert_spelling_with(
        &get_processor_with_tags(vec!["string"], vec![]),
        LanguageType::Markdown,
        MARKDOWN_WITH_PYTHON,
        &["strng"],
        &["commet", "calculat"],
    );
}

#[test]
fn test_text_language_ignores_tags() {
    // Text language doesn't use tree-sitter, so tags should have no effect;
    // Text mode checks everything regardless of tags.
    assert_spelling_with(
        &get_processor_with_tags(vec!["comment"], vec![]),
        LanguageType::Text,
        "This has a tset typo",
        &["tset"],
        &[],
    );
}
