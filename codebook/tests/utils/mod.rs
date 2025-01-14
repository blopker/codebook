use codebook::CodeDictionary;
static EXTRA_WORDS: &'static [&'static str] = &["http", "https", "www", "viewport", "UTF"];

pub fn get_processor() -> CodeDictionary {
    let mut cdict = CodeDictionary::new("./tests/en_index.aff", "./tests/en_index.dic").unwrap();
    for word in EXTRA_WORDS {
        cdict.add_to_dictionary(word);
    }
    cdict
}
