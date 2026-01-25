use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_yaml_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
      # On a sepaate line
      title: "Example lne"
      nested:
        name: Naame
        nested:
          item_name: Iteem name
      details: |
        # can be inented
        This is comented out
        this wors, too
      options:
        - opttions
        - parameters
      flags: froozen
      var: 'helo'
      symbol: ':hello'
    "#;
    let expected = vec![
        "Iteem", "Naame", "comented", "froozen", "helo", "inented", "lne", "opttions", "sepaate",
        "wors",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::YAML), None)
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
fn test_yaml_code() {
    utils::init_logging();
    let sample_yaml_code = r#"
      # On a separate line
      tiitle: "Example line"
      subtiitle: Subtitle
      descriptioon: 'hello'
      nested_struucture:
        name: "Name"
        nestted:
          item_name: 'Item name'
          another_name: Another Name
      options:
        - parameters:
        - parameters
      items: [ { id: 1, naame: "one" }, { id: 2, name: "two" } ]

    "#;

    let expected = vec![
        WordLocation::new(
            "tiitle".to_string(),
            vec![TextRange {
                start_byte: 34,
                end_byte: 40,
            }],
        ),
        WordLocation::new(
            "subtiitle".to_string(),
            vec![TextRange {
                start_byte: 63,
                end_byte: 72,
            }],
        ),
        WordLocation::new(
            "descriptioon".to_string(),
            vec![TextRange {
                start_byte: 89,
                end_byte: 101,
            }],
        ),
        WordLocation::new(
            "struucture".to_string(),
            vec![TextRange {
                start_byte: 124,
                end_byte: 134,
            }],
        ),
        WordLocation::new(
            "nestted".to_string(),
            vec![TextRange {
                start_byte: 165,
                end_byte: 172,
            }],
        ),
        WordLocation::new(
            "naame".to_string(),
            vec![TextRange {
                start_byte: 326,
                end_byte: 331,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_yaml_code, Some(LanguageType::YAML), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
}
