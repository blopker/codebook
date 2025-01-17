use codebook::{SpellCheckResult, TextRange};

mod utils;

#[test]
fn test_css_location() {
    let sample_css = r#"
        .test {
            color: red;
        }
        .testz {
            color: blue;
        }
"#;
    let expected = vec![SpellCheckResult::new(
        "testz".to_string(),
        vec![],
        vec![TextRange {
            start_char: 9,
            end_char: 14,
            start_line: 4,
            end_line: 4,
        }],
    )];
    let processor = utils::get_processor(false);
    let misspelled = processor.spell_check(sample_css, "css").to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}
