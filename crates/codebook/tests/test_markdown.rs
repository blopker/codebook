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
    let expected = vec![WordLocation::new(
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
fn test_markdown_fenced_code_block_skipped() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"# Hello World

Some correct text here.

```bash
mkdir some_dir
badwwword_in_code
```

More correct text here.
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Markdown), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // Words inside fenced code blocks should NOT be flagged
    assert!(!words.contains(&"mkdir"));
    assert!(!words.contains(&"badwwword"));
    assert!(!words.contains(&"dir"));
}

#[test]
fn test_markdown_fenced_code_block_with_typo_outside() {
    utils::init_logging();
    let processor = utils::get_processor();
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
    // Typos in prose should be flagged
    assert!(words.contains(&"paragrap"));
    assert!(words.contains(&"tyypo"));
    // Typos inside code blocks should NOT be flagged
    assert!(!words.contains(&"functin"));
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

```python
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
    assert!(!words.contains(&"mkdir"));
    assert!(!words.contains(&"somedir"));
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
