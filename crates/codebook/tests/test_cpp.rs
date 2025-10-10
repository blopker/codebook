use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};
mod utils;

#[test]
fn test_cpp_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        int calculatr(int numbr1, int numbr2, char operashun) {
            // This is an exampl function that performz calculashuns
            int resalt = 0;
            return resalt;
        }
    "#;
    let expected = vec![
        "calculashuns",
        "calculatr",
        "exampl",
        "numbr",
        "operashun",
        "performz",
        "resalt",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Cpp), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_cpp_comment_location() {
    utils::init_logging();
    let sample_cpp = r#"
        // Structur definition with misspellings
    "#;
    let expected = vec![WordLocation::new(
        "Structur".to_string(),
        vec![TextRange {
            start_byte: 12,
            end_byte: 20,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_cpp, Some(LanguageType::Cpp), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_cpp_class() {
    utils::init_logging();
    let sample_cpp = r#"
        class UserAccaunt {
            std::string usrrnamee;
            int ballancee;
            float intrest_rate;
        };
    "#;
    let expected = [
        WordLocation::new(
            "Accaunt".to_string(),
            vec![TextRange {
                start_byte: 19,
                end_byte: 26,
            }],
        ),
        WordLocation::new(
            "usrrnamee".to_string(),
            vec![TextRange {
                start_byte: 53,
                end_byte: 62,
            }],
        ),
        WordLocation::new(
            "ballancee".to_string(),
            vec![TextRange {
                start_byte: 80,
                end_byte: 89,
            }],
        ),
        WordLocation::new(
            "intrest".to_string(),
            vec![TextRange {
                start_byte: 109,
                end_byte: 116,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_cpp, Some(LanguageType::Cpp), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for expect in expected.iter() {
        println!("Expecting {}", expect.word);
        let result = misspelled.iter().find(|r| r.word == expect.word).unwrap();
        assert_eq!(result.word, expect.word);
        assert_eq!(result.locations, expect.locations);
    }
}

#[test]
fn test_cpp_multiline_string_concat() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        const char* message =
            "This is a verry long string\n"
            "that continuez on multiple linez\n"
            "with lots of speling misstakes\n";
    "#;
    let expected = vec!["continuez", "linez", "misstakes", "speling", "verry"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Cpp), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_cpp_vector_string_literals() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        std::vector<std::string> mesages = { "Helo", "Wrold", "Cpp" };
    "#;
    let expected = vec!["Helo", "Wrold", "mesages"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Cpp), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_cpp_stream_string_literal() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        std::cout << "Currect anser!\n youu best";
    "#;
    let expected = vec!["Currect", "anser", "youu"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Cpp), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}
