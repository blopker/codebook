use std::sync::Arc;

use codebook::dictionary::CodeDictionary;
static EXTRA_WORDS: &'static [&'static str] = &["http", "https", "www", "viewport", "UTF"];

pub fn get_processor() -> CodeDictionary {
    let config = Arc::new(codebook_config::CodebookConfig::default());
    let dict = CodeDictionary::new(config, "./tests/en_index.aff", "./tests/en_index.dic").unwrap();
    for word in EXTRA_WORDS {
        dict.add_to_dictionary(word);
    }
    dict
}

#[cfg(test)]
pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
