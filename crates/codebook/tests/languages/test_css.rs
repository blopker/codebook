use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};


#[test]
fn test_css_location() {
    super::utils::init_logging();
    let sample_css = r#"
        .test {
            color: red;
        }
        .testz {
            color: blue;
        }
"#;
    let expected = vec![WordLocation::new(
        "testz".to_string(),
        vec![TextRange {
            start_byte: 60,
            end_byte: 65,
        }],
    )];
    let processor = super::utils::get_processor();
    let misspelled = processor
        .spell_check(sample_css, Some(LanguageType::Css), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}
