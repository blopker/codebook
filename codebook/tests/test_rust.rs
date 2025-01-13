use codebook::{SpellCheckResult, TextRange};

mod utils;
// im a bd speler
#[test]
fn test_rust_simple() {
    let processor = utils::get_processor();
    let sample_text = r#"
        fn calculat_user_age(bithDate: String) -> u32 {
            // This is an examle_function that calculates age
            let usrAge = get_curent_date() - bithDate;
            userAge
        }
    "#;
    let expected = vec!["bith", "calculat", "examle", "usr"];
    let binding = processor.spell_check(sample_text, "rust").to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_rust_comment_location() {
    let sample_rust = r#"
        // Comment with a typo: mment
        "#;
    let expected = vec![SpellCheckResult::new(
        "mment".to_string(),
        vec!["moment", "comment", "Menkent"],
        vec![TextRange {
            start_char: 32,
            end_char: 37,
            start_line: 1,
            end_line: 1,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor.spell_check(sample_rust, "rust").to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}
