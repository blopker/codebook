use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};
mod utils;

#[test]
fn test_c_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        int calculatr(int numbr1, int numbr2, char operashun) {
            // This is an exampl function that performz calculashuns
            int resalt = 0;
            return resalt;
        }
    "#;
    let expected = vec![
        "calculashuns",
        "calculatr",
        "exampl",
        "numbr",
        "operashun",
        "performz",
        "resalt",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::C), None)
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
fn test_c_comment_location() {
    utils::init_logging();
    let sample_c = r#"
        // Structur definition with misspellings
    "#;
    let expected = vec![WordLocation::new(
        "Structur".to_string(),
        vec![TextRange {
            start_char: 11,
            end_char: 19,
            line: 1,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_c, Some(LanguageType::C), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_c_struct() {
    utils::init_logging();
    let sample_c = r#"
        struct UserAccaunt {
            char* usrrnamee;
            int ballancee;
            float intrest_rate;
        };
    "#;
    let expected = [
        WordLocation::new(
            "Accaunt".to_string(),
            vec![TextRange {
                start_char: 19,
                end_char: 26,
                line: 1,
            }],
        ),
        WordLocation::new(
            "usrrnamee".to_string(),
            vec![TextRange {
                start_char: 18,
                end_char: 27,
                line: 2,
            }],
        ),
        WordLocation::new(
            "ballancee".to_string(),
            vec![TextRange {
                start_char: 16,
                end_char: 25,
                line: 3,
            }],
        ),
        WordLocation::new(
            "intrest".to_string(),
            vec![TextRange {
                start_char: 18,
                end_char: 25,
                line: 4,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_c, Some(LanguageType::C), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for expect in expected.iter() {
        println!("Expecting {}", expect.word);
        let result = misspelled.iter().find(|r| r.word == expect.word).unwrap();
        assert_eq!(result.word, expect.word);
        assert_eq!(result.locations, expect.locations);
    }
}
