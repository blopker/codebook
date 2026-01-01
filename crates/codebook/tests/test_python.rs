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
fn test_python_import_statements() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        import no_typpoa
        import no_typpob.no_typpoc

        import no_typpod as yes_typpoe
        import no_typpof.no_typpog as yes_typpoh

        from no_typpoi import no_typpoj
        from no_typpok.no_typpol import no_typpom

        from no_typpoo import no_typpop as yes_typpoq
        from no_typpor.no_typpos import no_typpot as yes_typpou
        from .. import no_typpov as yes_typpow
    "#;
    let expected = vec!["typpoe", "typpoh", "typpoq", "typpou", "typpow"];
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
