use codebook::{Codebook, queries::LanguageType};
use codebook_config::{CodebookConfig, CodebookConfigMemory};
use std::sync::Arc;

fn get_processor_with_words(words: &[&str]) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    for w in words {
        let _ = config.add_word(w);
    }
    super::utils::make_codebook(config)
}

#[test]
fn test_custom_words() {
    let sample_text = r#"
        ok words
        testword
        good words
        actualbad
"#;
    // "testword" is added to the config dictionary, so only "actualbad" is flagged.
    super::utils::assert_spelling_with(
        &get_processor_with_words(&["testword"]),
        LanguageType::Text,
        sample_text,
        &["actualbad"],
        &["testword"],
    );
}
