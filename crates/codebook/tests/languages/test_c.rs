use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

#[test]
fn test_c_simple() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
        int calculatr(int numbr1, int numbr2, char operashun) {
            // This is an exampl function that performz calculashuns
            int resalt = 0;
            int misspellled;
            return resalt + misspellled;
        }
    "#;
    let expected = vec![
        "calculashuns",
        "calculatr",
        "exampl",
        "misspellled",
        "numbr",
        "operashun",
        "performz",
        "resalt",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::C), None)
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
fn test_c_comment_location() {
    super::utils::init_logging();
    let sample_c = r#"
        // Structur definition with misspellings
    "#;
    let expected = vec![WordLocation::new(
        "Structur".to_string(),
        vec![TextRange {
            start_byte: 12,
            end_byte: 20,
        }],
    )];
    let processor = super::utils::get_processor();
    let misspelled = processor
        .spell_check(sample_c, Some(LanguageType::C), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_c_struct() {
    super::utils::init_logging();
    let sample_c = r#"
        struct UserAccaunt {
            char* usrrnamee;
            int ballancee;
            float intrest_rate;
        };
    "#;
    let expected = [
        WordLocation::new(
            "Accaunt".to_string(),
            vec![TextRange {
                start_byte: 20,
                end_byte: 27,
            }],
        ),
        WordLocation::new(
            "usrrnamee".to_string(),
            vec![TextRange {
                start_byte: 48,
                end_byte: 57,
            }],
        ),
        WordLocation::new(
            "ballancee".to_string(),
            vec![TextRange {
                start_byte: 75,
                end_byte: 84,
            }],
        ),
        WordLocation::new(
            "intrest".to_string(),
            vec![TextRange {
                start_byte: 104,
                end_byte: 111,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let misspelled = processor
        .spell_check(sample_c, Some(LanguageType::C), None)
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
fn test_c_macros() {
    super::utils::init_logging();
    let sample_text = r#"
        #define MACROCONST 3
        #define MACROFUNC(macroparam) macroparam + 1
    "#;
    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "MACROCONST".to_string(),
            vec![TextRange {
                start_byte: 17,
                end_byte: 27,
            }],
        ),
        WordLocation::new(
            "MACROFUNC".to_string(),
            vec![TextRange {
                start_byte: 46,
                end_byte: 55,
            }],
        ),
        WordLocation::new(
            "macroparam".to_string(),
            vec![TextRange {
                start_byte: 56,
                end_byte: 66,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_unions() {
    super::utils::init_logging();
    let sample_text = r#"union myunion { int int_val; };"#;

    // Needs to be lexicographically sorted
    let expected = [WordLocation::new(
        "myunion".to_string(),
        vec![TextRange {
            start_byte: 6,
            end_byte: 13,
        }],
    )];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_variable_declarations() {
    super::utils::init_logging();
    let sample_text = r#"
        int arrayy[3];
        int* pointerr;
        int* pointerrarray[3];
        enum Role rolee;
        union Union unionn;
        struct User userr;"#;

    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "arrayy".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "pointerr".to_string(),
            vec![TextRange {
                start_byte: 37,
                end_byte: 45,
            }],
        ),
        WordLocation::new(
            "pointerrarray".to_string(),
            vec![TextRange {
                start_byte: 60,
                end_byte: 73,
            }],
        ),
        WordLocation::new(
            "rolee".to_string(),
            vec![TextRange {
                start_byte: 96,
                end_byte: 101,
            }],
        ),
        WordLocation::new(
            "unionn".to_string(),
            vec![TextRange {
                start_byte: 123,
                end_byte: 129,
            }],
        ),
        WordLocation::new(
            "userr".to_string(),
            vec![TextRange {
                start_byte: 151,
                end_byte: 156,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_variable_initializers() {
    super::utils::init_logging();
    // Note: variables with initializers have slightly different syntax tree
    // representations, so it useful to test them along with plain declarations.
    let sample_text = r#"
        int arrayy[3] = {};
        int* pointerr = NULL;
        int* pointerrarray[3] = {};
        enum Role rolee = ROLE1;
        union Union unionn = 10;
        struct User userr = {};"#;

    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "arrayy".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "pointerr".to_string(),
            vec![TextRange {
                start_byte: 42,
                end_byte: 50,
            }],
        ),
        WordLocation::new(
            "pointerrarray".to_string(),
            vec![TextRange {
                start_byte: 72,
                end_byte: 85,
            }],
        ),
        WordLocation::new(
            "rolee".to_string(),
            vec![TextRange {
                start_byte: 113,
                end_byte: 118,
            }],
        ),
        WordLocation::new(
            "unionn".to_string(),
            vec![TextRange {
                start_byte: 148,
                end_byte: 154,
            }],
        ),
        WordLocation::new(
            "userr".to_string(),
            vec![TextRange {
                start_byte: 181,
                end_byte: 186,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_field_declarations() {
    super::utils::init_logging();
    let sample_text = r#"
        struct MyStruct {
            int arrayy[3];
            int* pointerr;
            int* pointerrarray[3];
            enum Role rolee;
            union Union unionn;
            struct User userr;
        }"#;

    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "arrayy".to_string(),
            vec![TextRange {
                start_byte: 43,
                end_byte: 49,
            }],
        ),
        WordLocation::new(
            "pointerr".to_string(),
            vec![TextRange {
                start_byte: 71,
                end_byte: 79,
            }],
        ),
        WordLocation::new(
            "pointerrarray".to_string(),
            vec![TextRange {
                start_byte: 98,
                end_byte: 111,
            }],
        ),
        WordLocation::new(
            "rolee".to_string(),
            vec![TextRange {
                start_byte: 138,
                end_byte: 143,
            }],
        ),
        WordLocation::new(
            "unionn".to_string(),
            vec![TextRange {
                start_byte: 169,
                end_byte: 175,
            }],
        ),
        WordLocation::new(
            "userr".to_string(),
            vec![TextRange {
                start_byte: 201,
                end_byte: 206,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_strings() {
    super::utils::init_logging();

    // Note: we do not do spell checking across string concatenations,
    // just individual strings.
    let sample_text = r#"
        char* str1 = "aaaa bbbb";
        str1 = "cccc" "valid string" "dddd";
        printf("I'm a multiline stringg\n"
               "withh\nyyy");
    "#;

    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "aaaa".to_string(),
            vec![TextRange {
                start_byte: 23,
                end_byte: 27,
            }],
        ),
        WordLocation::new(
            "bbbb".to_string(),
            vec![TextRange {
                start_byte: 28,
                end_byte: 32,
            }],
        ),
        WordLocation::new(
            "cccc".to_string(),
            vec![TextRange {
                start_byte: 51,
                end_byte: 55,
            }],
        ),
        WordLocation::new(
            "dddd".to_string(),
            vec![TextRange {
                start_byte: 73,
                end_byte: 77,
            }],
        ),
        WordLocation::new(
            "stringg".to_string(),
            vec![TextRange {
                start_byte: 112,
                end_byte: 119,
            }],
        ),
        WordLocation::new(
            "withh".to_string(),
            vec![TextRange {
                start_byte: 139,
                end_byte: 144,
            }],
        ),
        WordLocation::new(
            "yyy".to_string(),
            vec![TextRange {
                start_byte: 146,
                end_byte: 149,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_typedef() {
    super::utils::init_logging();
    let sample_text = r#"typedef int Mispelll;"#;

    // Needs to be lexicographically sorted
    let expected = [WordLocation::new(
        "Mispelll".to_string(),
        vec![TextRange {
            start_byte: 12,
            end_byte: 20,
        }],
    )];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}

#[test]
fn test_c_enum() {
    super::utils::init_logging();
    let sample_text = r#"enum Colrs { Grean };"#;

    // Needs to be lexicographically sorted
    let expected = [
        WordLocation::new(
            "Colrs".to_string(),
            vec![TextRange {
                start_byte: 5,
                end_byte: 10,
            }],
        ),
        WordLocation::new(
            "Grean".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 18,
            }],
        ),
    ];
    let processor = super::utils::get_processor();
    let mut misspelled = processor.spell_check(sample_text, Some(LanguageType::C), None);
    misspelled.sort_by(|loc1, loc2| loc1.word.cmp(&loc2.word));
    assert_eq!(misspelled, expected);
}
