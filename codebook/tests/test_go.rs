use codebook::{SpellCheckResult, TextRange};

mod utils;

#[test]
fn test_go_location() {
    let sample_text = r#"
        func mispeledFuncion() string {
            return ""
        }"#;
    let expected = vec![
        SpellCheckResult::new(
            "mispeled".to_string(),
            vec!["misspelled", "dispelled", "misspell", "misled"],
            vec![TextRange {
                start_char: 13,
                end_char: 21,
                start_line: 1,
                end_line: 1,
            }],
        ),
        SpellCheckResult::new(
            "Funcion".to_string(),
            vec!["Function"],
            vec![TextRange {
                start_char: 21,
                end_char: 28,
                start_line: 1,
                end_line: 1,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor.spell_check(sample_text, "go").to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.suggestions, e.suggestions);
        assert_eq!(miss.locations, e.locations);
    }
}
