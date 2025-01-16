use codebook::{SpellCheckResult, TextRange};

mod utils;

#[test]
fn test_rust_simple() {
    utils::init_logging();
    let processor = utils::get_processor(true);
    let sample_text = r#"
        fn calculat_user_age(bithDate: String) -> u32 {
            // This is an examle_function that calculates age
            let usrAge = get_curent_date() - bithDate;
            userAge
        }
    "#;
    let expected = vec!["bith", "calculat", "examle"];
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
    utils::init_logging();
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
    let processor = utils::get_processor(true);
    let misspelled = processor.spell_check(sample_rust, "rust").to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_rust_struct() {
    utils::init_logging();
    let sample_rust = r#"
        pub struct BadSpeler {
            /// Terrible spelling: dwnloader
            pub dataz: String,
        }
        "#;
    let expected = vec![
        SpellCheckResult::new(
            "Speler".to_string(),
            vec!["Speer", "Speller", "Spewer", "Spengler", "Peeler"],
            vec![TextRange {
                start_char: 22,
                end_char: 28,
                start_line: 1,
                end_line: 1,
            }],
        ),
        SpellCheckResult::new(
            "dwnloader".to_string(),
            vec!["loader"],
            vec![TextRange {
                start_char: 35,
                end_char: 44,
                start_line: 2,
                end_line: 2,
            }],
        ),
        SpellCheckResult::new(
            "dataz".to_string(),
            vec!["data", "data z"],
            vec![TextRange {
                start_char: 16,
                end_char: 21,
                start_line: 3,
                end_line: 3,
            }],
        ),
    ];
    let processor = utils::get_processor(true);
    let misspelled = processor.spell_check(sample_rust, "rust").to_vec();
    println!("Misspelled words: {misspelled:?}");
    for expect in expected.iter() {
        println!("Expecting {}", expect.word);
        let result = misspelled.iter().find(|r| r.word == expect.word).unwrap();
        assert_eq!(result.word, expect.word);
        assert_eq!(result.suggestions, expect.suggestions);
        assert_eq!(result.locations, expect.locations);
    }
}
