use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

use crate::{
    assert_helpers::get_marked_misspellings,
    assert_word_locations_match,
    utils::{get_processor, get_sorted_misspellings},
};

#[test]
fn test_c_typedef() {
    super::utils::init_logging();
    let sample_text = r#"typedef int Mispelll;"#;

    // Needs to be lexicographically sorted
    let expected = [WordLocation::new(
        "Mispelll".to_string(),
        vec![TextRange {
            start_byte: 12,
            end_byte: 20,
        }],
    )];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_enum() {
    super::utils::init_logging();
    let sample_text = r#"enum Colrs { Grean };"#;

    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "Colrs".to_string(),
            vec![TextRange {
                start_byte: 5,
                end_byte: 10,
            }],
        ),
        WordLocation::new(
            "Grean".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 18,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_example_file() {
    let expected_result =
        get_marked_misspellings(include_str!("../examples/example.c.in"), "@@", "@@");

    super::utils::init_logging();
    let misspellings =
        get_sorted_misspellings(&expected_result.content, get_processor(), LanguageType::C);
    assert_word_locations_match!(misspellings, expected_result.misspellings);
}
