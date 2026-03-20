use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_markdown_paragraph() {
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    // bash builtins like mkdir should be recognized by the bash dictionary
    assert!(!words.contains(&"mkdir"));
    // dir is a common abbreviation, should not be flagged
    assert!(!words.contains(&"dir"));
}

#[test]
fn test_markdown_fenced_code_block_unknown_lang_skipped() {
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    utils::init_logging();
    let processor = utils::get_processor();
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
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // wrld should be flagged as a function name typo in both languages
    assert!(words.contains(&"wrld"));
}
