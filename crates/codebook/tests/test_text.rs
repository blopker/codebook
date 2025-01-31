use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

fn expect_spelling(text: &str, expected: &Vec<&str>) {
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(text, Some(LanguageType::Text), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, *expected);
}

#[test]
fn test_text_simple() {
    let sample_text = r#"
        I'm bvd at splellin Wolrd wolrd
        hello regular regu
    "#;
    let expected = vec!["Wolrd", "bvd", "regu", "splellin", "wolrd"];
    expect_spelling(sample_text, &expected);
}

#[test]
fn test_text_location() {
    let sample_text = r#"hello regular regu"#;
    let expected = vec![WordLocation::new(
        "regu".to_string(),
        vec![TextRange {
            start_char: 14,
            end_char: 18,
            line: 0,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}
