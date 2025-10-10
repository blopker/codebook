use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};
mod utils;

#[test]
fn test_elixir_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        defmodule Calculatr do
          # This is an exampl module that performz calculashuns
          def add(numbr1, numbr2) do
            resalt = numbr1 + numbr2
            resalt
          end
        end
    "#;
    let expected = vec![
        "Calculatr",
        "calculashuns",
        "exampl",
        "numbr",
        "performz",
        "resalt",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Elixir), None)
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
fn test_elixir_comment_location() {
    utils::init_logging();
    let sample_elixir = r#"
        # Structur definition with misspellings
    "#;
    let expected = vec![WordLocation::new(
        "Structur".to_string(),
        vec![TextRange {
            start_byte: 10,
            end_byte: 18,
        }],
    )];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_elixir, Some(LanguageType::Elixir), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    assert!(misspelled[0].locations.len() == 1);
}

#[test]
fn test_elixir_module() {
    utils::init_logging();
    let sample_elixir = r#"
        defmodule UserAccaunt do
          @moduledoc """
          This module handels user accaunts
          """

          defstruct [:usrrnamee, :ballancee, :intrest_rate]

          def create_accaunt(name) do
            %UserAccaunt{
              usrrnamee: name,
              ballancee: 0,
              intrest_rate: 0.05
            }
          end
        end
    "#;
    let expected = [
        WordLocation::new(
            "Accaunt".to_string(),
            vec![
                TextRange {
                    start_byte: 22,
                    end_byte: 29,
                },
                TextRange {
                    start_byte: 17,
                    end_byte: 24,
                },
            ],
        ),
        WordLocation::new(
            "handels".to_string(),
            vec![TextRange {
                start_byte: 22,
                end_byte: 29,
            }],
        ),
        WordLocation::new(
            "accaunts".to_string(),
            vec![TextRange {
                start_byte: 35,
                end_byte: 43,
            }],
        ),
        WordLocation::new(
            "usrrnamee".to_string(),
            vec![
                TextRange {
                    start_byte: 22,
                    end_byte: 31,
                },
                TextRange {
                    start_byte: 14,
                    end_byte: 23,
                },
            ],
        ),
        WordLocation::new(
            "ballancee".to_string(),
            vec![
                TextRange {
                    start_byte: 34,
                    end_byte: 43,
                },
                TextRange {
                    start_byte: 14,
                    end_byte: 23,
                },
            ],
        ),
        WordLocation::new(
            "intrest".to_string(),
            vec![
                TextRange {
                    start_byte: 46,
                    end_byte: 53,
                },
                TextRange {
                    start_byte: 14,
                    end_byte: 21,
                },
            ],
        ),
        WordLocation::new(
            "accaunt".to_string(),
            vec![TextRange {
                start_byte: 21,
                end_byte: 28,
            }],
        ),
    ];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_elixir, Some(LanguageType::Elixir), None)
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
fn test_elixir_functions() {
    utils::init_logging();
    let sample_elixir = r#"
        defmodule ProcessingPipeline do
          # Handles incomming data procesing
          def proccess_data(input) do
            input
            |> validatte()
            |> transfrom()
            |> persiste()
          end

          defp validatte(data) do
            # Performz validation logik
            data
          end

          defp transfrom(data) do
            # Aplies transformation ruls
            data
          end

          defp persiste(data) do
            # Savs to databse
            data
          end
        end
    "#;
    let expected = vec![
        "Aplies",
        "Performz",
        "Savs",
        "databse",
        "incomming",
        "logik",
        "persiste",
        "proccess",
        "procesing",
        "ruls",
        "transfrom",
        "validatte",
    ];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_elixir, Some(LanguageType::Elixir), None)
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
fn test_elixir_pattern_matching() {
    utils::init_logging();
    let sample_elixir = r#"
        defmodule PatternMatcher do
          def handle_responce({:ok, resalt}) do
            {:succes, resalt}
          end

          def handle_responce({:error, reson}) do
            {:failur, reson}
          end

          def parse_mesage(%{type: "notfication", conten: conten}) do
            process_notfication(conten)
          end
        end
    "#;
    let expected = vec![
        "conten",
        "failur",
        "mesage",
        "notfication",
        "responce",
        "succes",
    ];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_elixir, Some(LanguageType::Elixir), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}
