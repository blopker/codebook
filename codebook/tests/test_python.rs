use codebook::{SpellCheckResult, TextRange};

mod utils;

#[test]
fn test_python_simple() {
    let processor = utils::get_processor();
    let sample_text = r#"
        def calculat_user_age(bithDate: str) -> int:
            # This is an examle_function that calculates age
            usrAge = get_curent_date() - bithDate
            userAge
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

#[test]
fn test_python_multi_line_comment() {
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
                start_char: 57,
                end_char: 62,
                start_line: 2,
                end_line: 2,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor.spell_check(sample_python, "python").to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
}
