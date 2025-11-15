use codebook::{Codebook, queries::LanguageType};
use codebook_config::{CodebookConfigMemory, CustomDictionariesEntry};
use std::sync::Arc;

mod utils;
use crate::utils::example_file_path;

const CUSTOM_WORD: &'static str = "mycustomcorrectword";

pub fn get_processor(enable_custom_dict: bool) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());

    let custom_dict_name = "my_dict";
    let custom_dict_path = example_file_path("custom_dict.txt");
    let custom_dict = CustomDictionariesEntry {
        name: custom_dict_name.to_owned(),
        path: custom_dict_path,
        ..Default::default()
    };
    config.add_custom_dict(custom_dict);

    if enable_custom_dict {
        config.add_dict_id(&custom_dict_name);
    }

    Codebook::new(config.clone()).unwrap()
}

#[test]
fn test_custom_dict_unused_if_not_added_to_dicts() {
    let processor = get_processor(false);
    let misspelled = processor
        .spell_check(CUSTOM_WORD, Some(LanguageType::Text), None)
        .to_vec();

    assert_eq!(misspelled[0].word, CUSTOM_WORD);
}

#[test]
fn test_custom_dict_used_if_added_to_dicts() {
    let processor = get_processor(true);

    let misspelled = processor
        .spell_check(CUSTOM_WORD, Some(LanguageType::Text), None)
        .to_vec();

    // active custom dict
    assert!(misspelled.is_empty());
}
