use codebook::{SpellCheckResult, TextRange};

mod utils;

#[test]
fn test_toml_location() {
    let sample_toml = r#"
        name = "testx"
        [dependencies]
        toml = "0.5.8"
        testz = "0.1.0"
"#;
    let expected = vec![SpellCheckResult::new(
        "testx".to_string(),
        vec![],
        vec![TextRange {
            start_char: 16,
            end_char: 21,
            start_line: 1,
            end_line: 1,
        }],
    )];
    let not_expected = ["testz"];
    let processor = utils::get_processor(false);
    let misspelled = processor.spell_check(sample_toml, "toml").to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
