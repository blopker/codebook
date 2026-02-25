use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_vhdl_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
-- This is an exmple comment with speling errors
entity calculatr is
    port (
        clk     : in  std_logic;
        resett  : in  std_logic;
        inputt  : in  std_logic_vector(7 downto 0)
    );
end entity calculatr;
"#;
    let expected = vec![
        "calculatr",
        "clk",
        "exmple",
        "inputt",
        "resett",
        "speling",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::VHDL), None)
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
fn test_vhdl_comment_location() {
    utils::init_logging();
    let sample_text = r#"
-- A calculater for numbrs
"#;
    let expected = vec![
        WordLocation::new(
            "calculater".to_string(),
            vec![TextRange {
                start_byte: 6,
                end_byte: 16,
            }],
        ),
        WordLocation::new(
            "numbrs".to_string(),
            vec![TextRange {
                start_byte: 21,
                end_byte: 27,
            }],
        ),
    ];
    let not_expected = ["std_logic", "entity", "port", "signal"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::VHDL), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        assert!(miss.locations.len() == e.locations.len());
        for location in &miss.locations {
            assert!(e.locations.contains(location));
        }
    }
    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
