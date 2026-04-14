use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};


#[test]
fn test_markdown_paragraph() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "Some paragraph text with a misspeled word.\n";
    let expected = [WordLocation::new(
        "misspeled".to_string(),
        vec![TextRange {
            start_byte: 27,
            end_byte: 36,
        }],
    )];
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled.len(), 1);
    assert_eq!(misspelled[0].word, expected[0].word);
    assert_eq!(misspelled[0].locations, expected[0].locations);
}

#[test]
fn test_markdown_heading() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "# A headng with a tyypo\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"headng"));
    assert!(words.contains(&"tyypo"));
}

#[test]
fn test_markdown_fenced_code_block_known_lang() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Note: bash.scm only captures comments, strings, function names,
    // heredocs, and variable names, NOT command invocations.
    // So mkdir/some_dir are not checked because bash.scm doesn't capture them,
    // not because they're in a bash dictionary.
    let sample_text = r#"# Hello World

Some correct text here.

```bash
mkdir some_dir
```

More correct text here.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // bash.scm doesn't capture command invocations, so these are not checked
    assert!(!words.contains(&"mkdir"));
    assert!(!words.contains(&"dir"));
}

#[test]
fn test_markdown_fenced_code_block_unknown_lang_skipped() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"Some text.

```unknownlang
badwwword_in_code
```

More text.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // Unknown language code blocks are completely skipped
    assert!(!words.contains(&"badwwword"));
}

#[test]
fn test_markdown_fenced_code_block_no_lang_skipped() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"Some text.

```
badwwword_in_code
```

More text.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // Code blocks without language info are completely skipped
    assert!(!words.contains(&"badwwword"));
}

#[test]
fn test_markdown_code_block_uses_language_grammar() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // In Python grammar, function names are checked as identifiers
    let sample_text = r#"A paragrap with a tyypo.

```python
def some_functin():
    pass
```

Another paragrap with a tyypo.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // Prose typos should be flagged
    assert!(words.contains(&"paragrap"));
    assert!(words.contains(&"tyypo"));
    // Python function name typo should also be flagged (multi-language support!)
    assert!(words.contains(&"functin"));
}

#[test]
fn test_markdown_multiple_code_blocks() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"Some text with a tyypo.

```bash
mkdir somedir
```

Middle text is corect.

```unknownlang
badspel = True
```

End text is also corect.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"tyypo"));
    assert!(words.contains(&"corect"));
    // bash commands should be handled by bash grammar
    assert!(!words.contains(&"mkdir"));
    // unknown language blocks are skipped entirely
    assert!(!words.contains(&"badspel"));
}

#[test]
fn test_markdown_block_quote() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "> A block quoet with a tyypo.\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"quoet"));
    assert!(words.contains(&"tyypo"));
}

#[test]
fn test_markdown_code_block_alias_resolution() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Test that common aliases work (py -> Python, js -> Javascript, etc.)
    let sample_text = r#"Some text.

```py
def hello_wrld():
    pass
```

```js
function hello_wrld() {}
```

More text.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    // wrld should be flagged from both code blocks, verify two locations
    let wrld = misspelled.iter().find(|w| w.word == "wrld");
    assert!(wrld.is_some(), "wrld should be flagged");
    assert_eq!(
        wrld.unwrap().locations.len(),
        2,
        "wrld should have 2 locations (one from py block, one from js block)"
    );
}

#[test]
fn test_markdown_injected_region_byte_offsets() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Verify that byte offsets from injected regions map back correctly
    // to the original document coordinates.
    //                       0         1         2         3
    //                       0123456789012345678901234567890123456789
    let sample_text = "# OK\n\n```python\ndef some_functin(): pass\n```\n";
    //                       ^15 = start of python block content
    //                       "def some_functin(): pass\n" starts at byte 16
    //                       "functin" is at offset 9 within "def some_functin"
    //                       so global offset = 16 + 9 = 25
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    let functin = misspelled.iter().find(|w| w.word == "functin");
    assert!(functin.is_some(), "Expected 'functin' to be flagged");
    let loc = &functin.unwrap().locations[0];
    // Verify the byte offsets point to the right place in the original document
    assert_eq!(
        &sample_text[loc.start_byte..loc.end_byte],
        "functin",
        "Byte offsets should map back to 'functin' in the original document"
    );
}

#[test]
fn test_markdown_no_duplicate_spans() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Block quotes contain paragraphs. Make sure the inline content
    // isn't captured twice (once for the paragraph, once for the block quote)
    let sample_text = "> A tyypo in a block quoet.\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    for result in &misspelled {
        let unique_count = result.locations.len();
        let deduped: std::collections::HashSet<_> = result.locations.iter().collect();
        assert_eq!(
            unique_count,
            deduped.len(),
            "Word '{}' has duplicate spans: {:?}",
            result.word,
            result.locations
        );
    }
}
