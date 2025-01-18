use codebook::dictionary::{SpellCheckResult, TextRange};

mod utils;

#[test]
fn test_python_simple() {
    utils::init_logging();
    let processor = utils::get_processor(true);
    let sample_text = r#"
        def calculat_user_age(bithDate) -> int:
            # This is an examle_function that calculates age
            usrAge = get_curent_date() - bithDate
            userAge
    "#;
    let expected = vec!["bith", "calculat", "examle"];
    let binding = processor.spell_check(sample_text, "python").to_vec();
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
        SpellCheckResult::new(
            "mment".to_string(),
            vec!["moment", "comment", "Menkent"],
            vec![TextRange {
                start_char: 46,
                end_char: 51,
                start_line: 2,
                end_line: 2,
            }],
        ),
        SpellCheckResult::new(
            "linet".to_string(),
            vec![],
            vec![TextRange {
                start_char: 12,
                end_char: 17,
                start_line: 3,
                end_line: 3,
            }],
        ),
    ];
    let processor = utils::get_processor(true);
    let misspelled = processor.spell_check(sample_python, "python").to_vec();
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
        SpellCheckResult::new(
            "Spelin".to_string(),
            vec![],
            vec![TextRange {
                start_char: 9,
                end_char: 15,
                start_line: 1,
                end_line: 1,
            }],
        ),
        SpellCheckResult::new(
            "nospel".to_string(),
            vec![],
            vec![TextRange {
                start_char: 8,
                end_char: 14,
                start_line: 2,
                end_line: 2,
            }],
        ),
        SpellCheckResult::new(
            "hardz".to_string(),
            vec![],
            vec![TextRange {
                start_char: 28,
                end_char: 33,
                start_line: 5,
                end_line: 5,
            }],
        ),
        SpellCheckResult::new(
            "hardg".to_string(),
            vec![],
            vec![TextRange {
                start_char: 45,
                end_char: 50,
                start_line: 9,
                end_line: 9,
            }],
        ),
    ];
    let not_expected = vec!["zzzznomethod", "hardx", "hardd"];
    let processor = utils::get_processor(false);
    let misspelled = processor.spell_check(sample_python, "python").to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
    for word in not_expected {
        println!("Not expecting: {word:?}");
        assert!(misspelled.iter().find(|r| r.word == word).is_none());
    }
}
