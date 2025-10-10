use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_javascript_location() {
    utils::init_logging();
    let sample_text = r#"
    import { useState } from 'react';

    let objectz = {
        namee: "John",
        age: 30,
        city: "New York"
    };

    function calculaateScore(userInput) {
        const misspelleed = "thhis is wrong";
        let scoree = 0;

        // Check user input
        if (userInput.incluudes("test")) {
            scoree += 5;
        }
        try {
            // Some code that might throw an error
        } catch (errorz) {
            // Handle the error
        }
        return scoree + misspelleed.length;
    }"#;

    let expected = vec![
        WordLocation::new(
            "objectz".to_string(),
            vec![TextRange {
                start_byte: 8,
                end_byte: 15,
            }],
        ),
        WordLocation::new(
            "namee".to_string(),
            vec![TextRange {
                start_byte: 8,
                end_byte: 13,
            }],
        ),
        WordLocation::new(
            "calculaate".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 23,
            }],
        ),
        WordLocation::new(
            "misspelleed".to_string(),
            vec![TextRange {
                start_byte: 14,
                end_byte: 25,
            }],
        ),
        WordLocation::new(
            "thhis".to_string(),
            vec![TextRange {
                start_byte: 29,
                end_byte: 34,
            }],
        ),
        WordLocation::new(
            "scoree".to_string(),
            vec![TextRange {
                start_byte: 12,
                end_byte: 18,
            }],
        ),
        WordLocation::new(
            "errorz".to_string(),
            vec![TextRange {
                start_byte: 17,
                end_byte: 23,
            }],
        ),
    ];

    let not_expected = [
        "import",
        "useState",
        "react",
        "function",
        "const",
        "let",
        "if",
        "return",
        "length",
        "incluudes",
    ];

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Javascript), None)
        .to_vec();

    println!("Misspelled words: {misspelled:?}\n");

    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled
            .iter()
            .find(|r| r.word == e.word)
            .expect("Word not found");
        assert_eq!(miss.locations, e.locations);
    }

    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
