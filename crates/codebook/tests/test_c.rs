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
            start_byte: 12,
            end_byte: 20,
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
                start_byte: 20,
                end_byte: 27,
            }],
        ),
        WordLocation::new(
            "usrrnamee".to_string(),
            vec![TextRange {
                start_byte: 48,
                end_byte: 57,
            }],
        ),
        WordLocation::new(
            "ballancee".to_string(),
            vec![TextRange {
                start_byte: 75,
                end_byte: 84,
            }],
        ),
        WordLocation::new(
            "intrest".to_string(),
            vec![TextRange {
                start_byte: 104,
                end_byte: 111,
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
