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

#[test]
fn test_rust_trait_impl_function_names_not_checked() {
    // https://github.com/blopker/codebook/issues/225
    // Function names in `impl Trait for Type` blocks should not be spell-checked
    // because the names are dictated by the trait, not the implementor.
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        struct MyType;

        impl SomeTrait for MyType {
            fn spelling_erorr(self) {
                // This comment has a typo: tset
            }
        }
    "#;
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Rust), None)
        .to_vec();
    let misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {misspelled:?}");
    // "erorr" in the function name should NOT be flagged (name dictated by trait)
    assert!(
        !misspelled.contains(&"erorr"),
        "Function names in trait impl blocks should not be spell-checked"
    );
    // But comments inside the impl block should still be checked
    assert!(
        misspelled.contains(&"tset"),
        "Comments inside trait impl blocks should still be spell-checked"
    );
}

#[test]
fn test_rust_regular_impl_function_names_checked() {
    // Regular impl blocks (not trait implementations) should still be spell-checked
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        struct MyType;

        impl MyType {
            fn spelling_erorr(self) {}
        }

        fn top_level_erorr() {}
    "#;
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Rust), None)
        .to_vec();
    let misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {misspelled:?}");
    assert!(
        misspelled.contains(&"erorr"),
        "Expected 'erorr' to be flagged in regular impl and top-level functions"
    );
    let erorr = binding.iter().find(|r| r.word == "erorr").unwrap();
    assert_eq!(
        erorr.locations.len(),
        2,
        "Expected 'erorr' flagged in both regular impl and top-level function"
    );
}
