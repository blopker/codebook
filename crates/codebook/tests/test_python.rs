use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_python_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        def calculat_user_age(bithDate) -> int:
            # This is an examle_function that calculates age
            usrAge = get_curent_date() - bithDate
            userAge
    "#;
    let expected = vec!["bith", "calculat", "examle"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Python), None)
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
fn test_python_multi_line_comment() {
    utils::init_logging();
    let sample_python = r#"
multi_line_comment = '''
    This is a multi line comment with a typo: mment
    Another linet
'''
        "#;
    let expected = vec![
        WordLocation::new(
            "mment".to_string(),
            vec![TextRange {
                start_char: 46,
                end_char: 51,
                line: 2,
            }],
        ),
        WordLocation::new(
            "linet".to_string(),
            vec![TextRange {
                start_char: 12,
                end_char: 17,
                line: 3,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_python, Some(LanguageType::Python), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
}

#[test]
fn test_python_class() {
    utils::init_logging();
    let sample_python = r#"
class BadSpelin:
    def nospel(self):
        return self.zzzznomethod() # This should not get checked
    def bad_spelin(self): # This should get checked
        return "Spelling is hardz" # This should get checked

@decorated
def constructor():
    return BadSpelin(hardx=bad.hardd, thing="hardg")  # Some of this should get checked
'''
        "#;
    let expected = vec![
        WordLocation::new(
            "Spelin".to_string(),
            vec![TextRange {
                start_char: 9,
                end_char: 15,
                line: 1,
            }],
        ),
        WordLocation::new(
            "nospel".to_string(),
            vec![TextRange {
                start_char: 8,
                end_char: 14,
                line: 2,
            }],
        ),
        WordLocation::new(
            "hardz".to_string(),
            vec![TextRange {
                start_char: 28,
                end_char: 33,
                line: 5,
            }],
        ),
        WordLocation::new(
            "hardg".to_string(),
            vec![TextRange {
                start_char: 45,
                end_char: 50,
                line: 9,
            }],
        ),
    ];
    let not_expected = vec!["zzzznomethod", "hardx", "hardd"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_python, Some(LanguageType::Python), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
    for word in not_expected {
        println!("Not expecting: {word:?}");
        assert!(!misspelled.iter().any(|r| r.word == word));
    }
}

#[test]
fn test_python_global_variables() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
# Globul variables
globalCountr = 0
mesage = "Helllo Wolrd!"
    "#;
    let expected = vec![
        WordLocation::new(
            "Globul".to_string(),
            vec![TextRange {
                start_char: 2,
                end_char: 8,
                line: 1,
            }],
        ),
        WordLocation::new(
            "Countr".to_string(),
            vec![TextRange {
                start_char: 6,
                end_char: 12,
                line: 2,
            }],
        ),
        WordLocation::new(
            "mesage".to_string(),
            vec![TextRange {
                start_char: 0,
                end_char: 6,
                line: 3,
            }],
        ),
        WordLocation::new(
            "Helllo".to_string(),
            vec![TextRange {
                start_char: 10,
                end_char: 16,
                line: 3,
            }],
        ),
        WordLocation::new(
            "Wolrd".to_string(),
            vec![TextRange {
                start_char: 17,
                end_char: 22,
                line: 3,
            }],
        ),
    ];

    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Python), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");

    for e in &expected {
        let miss = misspelled
            .iter()
            .find(|r| r.word == e.word)
            .unwrap_or_else(|| panic!("Word '{}' not found in misspelled list", e.word));
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
}
