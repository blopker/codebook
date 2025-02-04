use std::sync::Arc;

use codebook::Codebook;

pub fn get_processor() -> Codebook {
    let config = Arc::new(codebook_config::CodebookConfig::new_no_file());
    let dict = Codebook::new(config).unwrap();
    dict
}

#[test]
fn test_suggestions() {
    let processor = get_processor();
    let suggestions = processor.get_suggestions("testz");
    println!("Suggestion words: {suggestions:?}");
    assert!(suggestions.len() != 0);
}
