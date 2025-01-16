use codebook::{SpellCheckResult, TextRange};

mod utils;

fn expect_spelling(text: &str, expected: &Vec<&str>) {
    let processor = utils::get_processor(true);
    let binding = processor.spell_check(text, "text").to_vec();
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
    let expected = vec![SpellCheckResult::new(
        "regu".to_string(),
        vec!["reg", "reg u"],
        vec![TextRange {
            start_char: 14,
            end_char: 18,
            start_line: 0,
            end_line: 0,
        }],
    )];
    let processor = utils::get_processor(true);
    let misspelled = processor.spell_check(sample_text, "text").to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}
