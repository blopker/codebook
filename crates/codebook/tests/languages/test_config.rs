use codebook::{
    Codebook,
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};
use codebook_config::{CodebookConfig, CodebookConfigMemory};
use std::sync::Arc;

pub fn get_processor(words: Option<&[&str]>) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    if let Some(words) = words {
        for w in words {
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
            start_byte: 62,
            end_byte: 71,
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
