use std::sync::Arc;

use codebook::Codebook;

pub fn get_processor() -> Codebook {
    let config = Arc::new(codebook_config::CodebookConfig::default());
    Codebook::new(config).unwrap()
}

#[test]
fn test_suggestions() {
    let processor = get_processor();
    let suggestions = processor.get_suggestions("testz");
    println!("Suggestion words: {suggestions:?}");
    assert!(!suggestions.unwrap().is_empty());
}
