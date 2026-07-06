use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_erlang_simple() {
    let sample_text = r#"
        -module(calculatr).
        % This is an exampl module that performz calculashuns
        -export([add/2]).

        add(Numbr1, Numbr2) ->
            Resalt = Numbr1 + Numbr2,
            Resalt.
    "#;
    assert_spelling_at(
        LanguageType::Erlang,
        sample_text,
        &[
            ("calculatr", &[0]),
            ("exampl", &[0]),
            ("performz", &[0]),
            ("calculashuns", &[0]),
            // Erlang variables are flagged at every occurrence, not just
            // where they are bound.
            ("Numbr", &[0, 1, 2, 3]),
            ("Resalt", &[0, 1]),
        ],
    );
}

#[test]
fn test_erlang_comment_location() {
    assert_spelling(
        LanguageType::Erlang,
        "\n        % Structur definition with misspellings\n    ",
        &["Structur"],
        &[],
    );
}

#[test]
fn test_erlang_pattern_matching() {
    let sample_text = r#"
        -module(example).
        -export([handle_response/1]).

        handle_response({ok, Resalt}) ->
            {succes, Resalt};
        handle_response({error, Reson}) ->
            {failur, Reson}.

        parse_message(#{type := <<"notfication">>, conten := Conten}) ->
            process_notfication(Conten).
    "#;
    assert_spelling_at(
        LanguageType::Erlang,
        sample_text,
        &[
            // Variables are flagged at every occurrence (pattern and body).
            ("Resalt", &[0, 1]),
            ("Reson", &[0, 1]),
            ("Conten", &[0, 1]),
            ("succes", &[0]),
            ("failur", &[0]),
            ("conten", &[0]),
            // Flagged in the binary string and again inside the
            // process_notfication atom.
            ("notfication", &[0, 1]),
        ],
    );
}
