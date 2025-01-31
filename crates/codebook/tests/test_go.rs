use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_go_location() {
    utils::init_logging();
    let sample_text = r#"
    import (
        "fmt"
    )
    func mispeledFuncion() string {
        var alicz = "Alizzz"
        return alicz
    }"#;
    let expected = vec![
        WordLocation::new(
            "mispeled".to_string(),
            vec![TextRange {
                start_char: 9,
                end_char: 17,
                line: 4,
            }],
        ),
        WordLocation::new(
            "Funcion".to_string(),
            vec![TextRange {
                start_char: 17,
                end_char: 24,
                line: 4,
            }],
        ),
        WordLocation::new(
            "Alizzz".to_string(),
            vec![TextRange {
                start_char: 21,
                end_char: 27,
                line: 5,
            }],
        ),
        WordLocation::new(
            "alicz".to_string(),
            vec![TextRange {
                start_char: 12,
                end_char: 17,
                line: 5,
            }],
        ),
    ];
    let not_expected = ["fmt"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Go), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        assert_eq!(miss.locations, e.locations);
    }
    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
