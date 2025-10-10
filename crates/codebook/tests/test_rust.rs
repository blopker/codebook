use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};
mod utils;

#[test]
fn test_rust_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        fn calculat_user_age(bithDate: String) -> u32 {
            // This is an examle_function that calculates age
            let usrAge = get_curent_date() - bithDate;
            userAge
        }
    "#;
    let expected = vec!["bith", "calculat", "examle"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Rust), None)
        .to_vec();
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
    let expected = vec![WordLocation::new(
        "mment".to_string(),
        vec![TextRange {
            start_byte: 33,
            end_byte: 38,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_rust, Some(LanguageType::Rust), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_rust_block_comments() {
    utils::init_logging();
    let sample_rust = r#"
        /* Comment with a typos on multiple lines: mment

        examle
        */

        /*! Inner block doc comment: testz
        */

        /** Outer block doc comment.

        Eror.
        */
        "#;
    let expected = [
        WordLocation::new(
            "mment".to_string(),
            vec![TextRange {
                start_byte: 52,
                end_byte: 57,
            }],
        ),
        WordLocation::new(
            "examle".to_string(),
            vec![TextRange {
                start_byte: 67,
                end_byte: 73,
            }],
        ),
        WordLocation::new(
            "testz".to_string(),
            vec![TextRange {
                start_byte: 123,
                end_byte: 128,
            }],
        ),
        WordLocation::new(
            "Eror".to_string(),
            vec![TextRange {
                start_byte: 187,
                end_byte: 191,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_rust, Some(LanguageType::Rust), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for expect in expected.iter() {
        println!("Expecting {}", expect.word);
        let result = misspelled.iter().find(|r| r.word == expect.word).unwrap();
        assert_eq!(result.word, expect.word);
        assert_eq!(result.locations, expect.locations);
    }
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
    let expected = [
        WordLocation::new(
            "Speler".to_string(),
            vec![TextRange {
                start_byte: 23,
                end_byte: 29,
            }],
        ),
        WordLocation::new(
            "dwnloader".to_string(),
            vec![TextRange {
                start_byte: 67,
                end_byte: 76,
            }],
        ),
        WordLocation::new(
            "dataz".to_string(),
            vec![TextRange {
                start_byte: 93,
                end_byte: 98,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_rust, Some(LanguageType::Rust), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for expect in expected.iter() {
        println!("Expecting {}", expect.word);
        let result = misspelled.iter().find(|r| r.word == expect.word).unwrap();
        assert_eq!(result.word, expect.word);
        assert_eq!(result.locations, expect.locations);
    }
}
