use codebook::queries::LanguageType;

use crate::{
    assert_helpers::get_marked_misspellings,
    assert_word_locations_match,
    utils::{get_processor, get_sorted_misspellings},
};

#[test]
fn test_c_example_file() {
    let expected_result =
        get_marked_misspellings(include_str!("../examples/example.c.in"), "@@", "@@");

    super::utils::init_logging();
    let misspellings =
        get_sorted_misspellings(&expected_result.content, get_processor(), LanguageType::C);
    assert_word_locations_match!(misspellings, expected_result.misspellings);
}

#[test]
fn test_c_type_uses() {
    super::utils::init_logging();
    let sample_text = r#"
        enum Colorr color;
        union Dataa data;
        struct Userr user;
    "#;

    let processor = super::utils::get_processor();
    let misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    assert_eq!(misspelled, []);
}
