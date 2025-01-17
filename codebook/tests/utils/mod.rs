use codebook::CodeDictionary;
static EXTRA_WORDS: &'static [&'static str] = &["http", "https", "www", "viewport", "UTF"];

pub fn get_processor(make_suggestions: bool) -> CodeDictionary {
    let config = codebook_config::CodebookConfig::default();
    let mut dict =
        CodeDictionary::new(config, "./tests/en_index.aff", "./tests/en_index.dic").unwrap();
    dict.make_suggestions = make_suggestions;
    for word in EXTRA_WORDS {
        dict.add_to_dictionary(word);
    }
    dict
}

pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}
