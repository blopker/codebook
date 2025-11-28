use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

/// Strategy:
/// Use distinct misspellings to test location sensitive checking.
/// This simpler to write than asserting exact locations.
/// Granted - it doesn't test that the spell_check return correct locations,
/// but should be sufficient that some tests tests this.
/// This can be used to test language-specific grammar rules with less effort.
///
/// `not_expected` does not have to be exhaustive.
fn assert_simple_misspellings(
    processor: &codebook::Codebook,
    sample_text: &str,
    expected_misspellings: Vec<&str>,
    not_expected: Vec<&str>,
    language: LanguageType,
) {
    // Check that the misspelled words used are distinct,
    // otherwise the test could fail to properly test location sensitive properties
    for word in expected_misspellings.iter() {
        let count = sample_text.matches(word).count();
        assert_eq!(
            count, 1,
            "Word '{}' should occur exactly once in sample_text, but found {} occurrences",
            word, count
        );
    }

    let binding = processor
        .spell_check(sample_text, Some(language), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    let mut expected_misspellings_sorted = expected_misspellings.clone();
    expected_misspellings_sorted.sort();
    assert_eq!(misspelled, expected_misspellings_sorted);

    for word in not_expected {
        println!("Not expecting: {word:?}");
        assert!(!misspelled.iter().any(|w| *w == word));
    }
}

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
                start_byte: 72,
                end_byte: 77,
            }],
        ),
        WordLocation::new(
            "linet".to_string(),
            vec![TextRange {
                start_byte: 90,
                end_byte: 95,
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
                start_byte: 10,
                end_byte: 16,
            }],
        ),
        WordLocation::new(
            "nospel".to_string(),
            vec![TextRange {
                start_byte: 26,
                end_byte: 32,
            }],
        ),
        WordLocation::new(
            "hardz".to_string(),
            vec![TextRange {
                start_byte: 185,
                end_byte: 190,
            }],
        ),
        WordLocation::new(
            "hardg".to_string(),
            vec![TextRange {
                start_byte: 294,
                end_byte: 299,
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
                start_byte: 3,
                end_byte: 9,
            }],
        ),
        WordLocation::new(
            "Countr".to_string(),
            vec![TextRange {
                start_byte: 26,
                end_byte: 32,
            }],
        ),
        WordLocation::new(
            "mesage".to_string(),
            vec![TextRange {
                start_byte: 37,
                end_byte: 43,
            }],
        ),
        WordLocation::new(
            "Helllo".to_string(),
            vec![TextRange {
                start_byte: 47,
                end_byte: 53,
            }],
        ),
        WordLocation::new(
            "Wolrd".to_string(),
            vec![TextRange {
                start_byte: 54,
                end_byte: 59,
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

#[test]
fn test_python_f_strings() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
name = "John"
age = 25
message = f'Hello, my naem is {namz} and I am {age} years oldd'
another = f"This is antoher examle with {name} varibles"
simple = f'check these wordz {but} {not} {the} {variables}'
    "#;

    let expected = vec![
        WordLocation::new(
            "naem".to_string(),
            vec![TextRange {
                start_byte: 46,
                end_byte: 50,
            }],
        ),
        WordLocation::new(
            "oldd".to_string(),
            vec![TextRange {
                start_byte: 82,
                end_byte: 86,
            }],
        ),
        WordLocation::new(
            "antoher".to_string(),
            vec![TextRange {
                start_byte: 108,
                end_byte: 115,
            }],
        ),
        WordLocation::new(
            "examle".to_string(),
            vec![TextRange {
                start_byte: 116,
                end_byte: 122,
            }],
        ),
        WordLocation::new(
            "varibles".to_string(),
            vec![TextRange {
                start_byte: 135,
                end_byte: 143,
            }],
        ),
        WordLocation::new(
            "wordz".to_string(),
            vec![TextRange {
                start_byte: 168,
                end_byte: 173,
            }],
        ),
    ];

    let not_expected = vec!["namz", "age", "but", "not", "the", "variables"];

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

    for word in not_expected {
        println!("Not expecting: {word:?}");
        assert!(!misspelled.iter().any(|r| r.word == word));
    }
}

#[test]
fn test_python_functions() {
    utils::init_logging();
    let processor = utils::get_processor();

    // Test simple function - function name and parameter names should be checked
    let simple_function = r#"
def simple_wrngfunction_name(wrngparam, correct, wrngdefaultparam=1, correct_default=2):
    pass
    "#;
    assert_simple_misspellings(
        &processor,
        simple_function,
        vec!["wrngfunction", "wrngparam", "wrngdefaultparam"],
        vec!["simple", "correct", "def", "name", "default"],
        LanguageType::Python,
    );

    // Test typed function - function names and parameters should be checked, but not types or modules
    let simple_typed_function = r#"
def simple_wrngfunction(wrngparam: str, correct: Wrngtype, other: wrngmod.Wrngmodtype, correct_default: Nons | int = 2) -> Wrngret:
    pass
    "#;
    assert_simple_misspellings(
        &processor,
        simple_typed_function,
        vec!["wrngfunction", "wrngparam"],
        vec![
            "simple",
            "correct",
            "str",
            "Wrngtype",
            "wrngmod",
            "Wrngmodtype",
            "Wrngret",
            "def",
            "Nons",
            "default",
        ],
        LanguageType::Python,
    );

    // Test generic function 1 - function names and parameters should be checked, but not types
    let generic_function_1 = r#"
def simple_wrngfunction(wrngparam: str, correct: Wrngtype[Wrngtemplate]):
    pass
    "#;
    assert_simple_misspellings(
        &processor,
        generic_function_1,
        vec!["wrngfunction", "wrngparam"],
        vec!["simple", "correct", "str", "Wrngtype", "Wrngtemplate"],
        LanguageType::Python,
    );

    // Test generic function 2 - function names and parameters should be checked, but not type templates
    let generic_function_2 = r#"
def simple_wrngfunction[Wrgtemplate](wrngparam: str, correct: Wrngtype[Wrngtemplate]):
    pass
    "#;
    assert_simple_misspellings(
        &processor,
        generic_function_2,
        vec!["wrngfunction", "wrngparam"],
        vec![
            "simple",
            "correct",
            "str",
            "Wrgtemplate",
            "Wrngtype",
            "Wrngtemplate",
        ],
        LanguageType::Python,
    );
}
