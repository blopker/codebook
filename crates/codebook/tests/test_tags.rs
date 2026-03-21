use codebook::queries::LanguageType;
mod utils;

/// Sample Rust code with misspellings in comments, strings, and identifiers.
const RUST_SAMPLE: &str = r#"
    // A commet with a typo
    fn calculat_age() {
        let nmber = "a strng value";
    }
"#;

fn check(text: &str, lang: LanguageType, include: Vec<&str>, exclude: Vec<&str>) -> Vec<String> {
    let processor = utils::get_processor_with_tags(include, exclude);
    let mut words: Vec<String> = processor
        .spell_check(text, Some(lang), None)
        .iter()
        .map(|r| r.word.clone())
        .collect();
    words.sort();
    words
}

#[test]
fn test_no_filters_returns_all() {
    let words = check(RUST_SAMPLE, LanguageType::Rust, vec![], vec![]);
    // Should find typos in all three categories
    assert!(
        words.contains(&"commet".to_string()),
        "missing comment typo"
    );
    assert!(
        words.contains(&"calculat".to_string()),
        "missing identifier typo"
    );
    assert!(words.contains(&"strng".to_string()), "missing string typo");
}

#[test]
fn test_include_comments_only() {
    let words = check(RUST_SAMPLE, LanguageType::Rust, vec!["comment"], vec![]);
    assert!(
        words.contains(&"commet".to_string()),
        "missing comment typo"
    );
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier should be excluded"
    );
    assert!(
        !words.contains(&"strng".to_string()),
        "string should be excluded"
    );
    assert!(
        !words.contains(&"nmber".to_string()),
        "variable should be excluded"
    );
}

#[test]
fn test_include_strings_only() {
    let words = check(RUST_SAMPLE, LanguageType::Rust, vec!["string"], vec![]);
    assert!(words.contains(&"strng".to_string()), "missing string typo");
    assert!(
        !words.contains(&"commet".to_string()),
        "comment should be excluded"
    );
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier should be excluded"
    );
}

#[test]
fn test_include_identifiers_only() {
    let words = check(RUST_SAMPLE, LanguageType::Rust, vec!["identifier"], vec![]);
    assert!(
        words.contains(&"calculat".to_string()),
        "missing function name typo"
    );
    assert!(
        words.contains(&"nmber".to_string()),
        "missing variable name typo"
    );
    assert!(
        !words.contains(&"commet".to_string()),
        "comment should be excluded"
    );
    assert!(
        !words.contains(&"strng".to_string()),
        "string should be excluded"
    );
}

#[test]
fn test_exclude_identifiers() {
    let words = check(RUST_SAMPLE, LanguageType::Rust, vec![], vec!["identifier"]);
    assert!(
        words.contains(&"commet".to_string()),
        "missing comment typo"
    );
    assert!(words.contains(&"strng".to_string()), "missing string typo");
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier should be excluded"
    );
    assert!(
        !words.contains(&"nmber".to_string()),
        "variable should be excluded"
    );
}

#[test]
fn test_exclude_specific_subtag() {
    // Exclude only identifier.variable, keep identifier.function
    let words = check(
        RUST_SAMPLE,
        LanguageType::Rust,
        vec![],
        vec!["identifier.variable"],
    );
    assert!(
        words.contains(&"calculat".to_string()),
        "function name should still be checked"
    );
    assert!(
        !words.contains(&"nmber".to_string()),
        "variable should be excluded"
    );
}

#[test]
fn test_include_and_exclude_combined() {
    // Include comments and strings, but exclude string specifically
    let words = check(
        RUST_SAMPLE,
        LanguageType::Rust,
        vec!["comment", "string"],
        vec!["string"],
    );
    assert!(
        words.contains(&"commet".to_string()),
        "missing comment typo"
    );
    assert!(
        !words.contains(&"strng".to_string()),
        "string should be excluded by exclude_tags"
    );
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier not in include_tags"
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
    let words = check(MARKDOWN_WITH_PYTHON, LanguageType::Markdown, vec![], vec![]);
    assert!(
        words.contains(&"commet".to_string()),
        "missing comment typo from injected python"
    );
    assert!(
        words.contains(&"calculat".to_string()),
        "missing identifier typo from injected python"
    );
    assert!(
        words.contains(&"strng".to_string()),
        "missing string typo from injected python"
    );
}

#[test]
fn test_injection_include_comments_only() {
    // include_tags = ["comment"] should only check comments,
    // even inside injected code blocks
    let words = check(
        MARKDOWN_WITH_PYTHON,
        LanguageType::Markdown,
        vec!["comment"],
        vec![],
    );
    assert!(
        words.contains(&"commet".to_string()),
        "comment typo should be found"
    );
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier should be excluded in injected region"
    );
    assert!(
        !words.contains(&"strng".to_string()),
        "string should be excluded in injected region"
    );
}

#[test]
fn test_injection_exclude_identifiers() {
    // exclude_tags = ["identifier"] should suppress identifiers
    // in both prose and injected code blocks
    let words = check(
        MARKDOWN_WITH_PYTHON,
        LanguageType::Markdown,
        vec![],
        vec!["identifier"],
    );
    assert!(
        words.contains(&"commet".to_string()),
        "comment should still be checked"
    );
    assert!(
        words.contains(&"strng".to_string()),
        "string should still be checked"
    );
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier should be excluded in injected region"
    );
}

#[test]
fn test_injection_include_strings_only() {
    // include_tags = ["string"] should check strings in both
    // markdown prose (which uses @string) and injected python
    let words = check(
        MARKDOWN_WITH_PYTHON,
        LanguageType::Markdown,
        vec!["string"],
        vec![],
    );
    assert!(
        words.contains(&"strng".to_string()),
        "string typo in injected python should be found"
    );
    assert!(
        !words.contains(&"commet".to_string()),
        "comment should be excluded"
    );
    assert!(
        !words.contains(&"calculat".to_string()),
        "identifier should be excluded"
    );
}

#[test]
fn test_text_language_ignores_tags() {
    // Text language doesn't use tree-sitter, so tags should have no effect
    let processor = utils::get_processor_with_tags(vec!["comment"], vec![]);
    let text = "This has a tset typo";
    let words: Vec<String> = processor
        .spell_check(text, Some(LanguageType::Text), None)
        .iter()
        .map(|r| r.word.clone())
        .collect();
    assert!(
        words.contains(&"tset".to_string()),
        "Text mode should check everything regardless of tags"
    );
}
