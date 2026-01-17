use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};
mod utils;

#[test]
fn test_erlang_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        -module(calculatr).
        % This is an exampl module that performz calculashuns
        -export([add/2]).

        add(Numbr1, Numbr2) ->
            Resalt = Numbr1 + Numbr2,
            Resalt.
    "#;
    let expected = vec![
        "Numbr",
        "Resalt",
        "calculashuns",
        "calculatr",
        "exampl",
        "performz",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Erlang), None)
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
fn test_erlang_comment_location() {
    utils::init_logging();
    let sample_erlang = r#"
        % Structur definition with misspellings
    "#;
    let expected = vec![WordLocation::new(
        "Structur".to_string(),
        vec![TextRange {
            start_byte: 11,
            end_byte: 19,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_erlang, Some(LanguageType::Erlang), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_erlang_pattern_matching() {
    utils::init_logging();
    let sample_erlang = r#"
        -module(example).
        -export([handle_response/1]).

        handle_response({ok, Resalt}) ->
            {succes, Resalt};
        handle_response({error, Reson}) ->
            {failur, Reson}.

        parse_message(#{type := <<"notfication">>, conten := Conten}) ->
            process_notfication(Conten).
    "#;
    let expected = vec![
        "Conten",
        "Resalt",
        "Reson",
        "conten",
        "failur",
        "notfication",
        "succes",
    ];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_erlang, Some(LanguageType::Erlang), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}
