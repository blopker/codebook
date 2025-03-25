use std::sync::Arc;

use codebook::{
    Codebook,
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

pub fn get_processor(words: Option<&[&str]>) -> Codebook {
    let config = Arc::new(codebook_config::CodebookConfig::default());
    if words.is_some() {
        for w in words.unwrap() {
            let _ = config.add_word(w);
        }
    }
    Codebook::new(config).unwrap()
}

#[test]
fn test_custom_words() {
    let sample_text = r#"
        ok words
        testword
        good words
        actualbad
"#;
    let expected = vec![WordLocation::new(
        "actualbad".to_string(),
        vec![TextRange {
            start_char: 8,
            end_char: 17,
            line: 4,
        }],
    )];
    let not_expected = ["testword"];
    let processor = get_processor(Some(&not_expected));
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
