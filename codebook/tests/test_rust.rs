use codebook::CodeDictionary;
static EXTRA_WORDS: &'static [&'static str] = &["http", "https", "www", "viewport", "UTF"];

fn get_processor() -> CodeDictionary {
    let mut cdict = CodeDictionary::new("./tests/en_index.aff", "./tests/en_index.dic").unwrap();
    for word in EXTRA_WORDS {
        cdict.add_to_dictionary(word.to_string());
    }
    cdict
}

#[test]
fn test_programming() {
    let processor = get_processor();
    let sample_text = r#"
        fn calculat_user_age(bithDate: String) -> u32 {
            // This is an examle_function that calculates age
            let usrAge = get_curent_date() - bithDate;
            userAge
        }
    "#;
    let expected = vec!["bith", "calculat", "examle", "usr"];
    let binding = processor.spell_check(sample_text, "rust").to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}
