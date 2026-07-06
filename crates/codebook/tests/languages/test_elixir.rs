use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_elixir_simple() {
    let sample_text = r#"
        defmodule Calculatr do
          # This is an exampl module that performz calculashuns
          def add(numbr1, numbr2) do
            resalt = numbr1 + numbr2
            resalt
          end
        end
    "#;
    // "numbr" is flagged at the two parameter definitions only; the usages
    // in `numbr1 + numbr2` are not definitions. "resalt" is flagged at the
    // `=` binding, not at the bare return usage.
    assert_spelling_at(
        LanguageType::Elixir,
        sample_text,
        &[
            ("Calculatr", &[0]),
            ("exampl", &[0]),
            ("performz", &[0]),
            ("calculashuns", &[0]),
            ("numbr", &[0, 1]),
            ("resalt", &[0]),
        ],
    );
}

#[test]
fn test_elixir_binary_operator_bindings() {
    // Only `=` and comprehension generators (`<-`) bind variables. The left
    // side of any other binary operator (arithmetic, `|>`, ...) is a usage
    // and must not be flagged.
    let sample_text = r#"
        defmodule Sample do
          def process(itemms) do
            for itemm <- itemms, do: itemm * 2
          end

          def pipeline(inputt) do
            inputt |> IO.inspect()
          end
        end
    "#;
    assert_spelling_at(
        LanguageType::Elixir,
        sample_text,
        &[
            // Parameter definition only, not the `<- itemms` usage.
            ("itemms", &[0]),
            // The `<-` generator binding; occurrence 0 is inside "itemms",
            // occurrence 3 is the `itemm * 2` usage.
            ("itemm", &[1]),
            // Parameter definition only, not the `inputt |>` usage.
            ("inputt", &[0]),
        ],
    );
}

#[test]
fn test_elixir_comment_location() {
    let sample_elixir = r#"
        # Structur definition with misspellings
    "#;
    assert_spelling(LanguageType::Elixir, sample_elixir, &["Structur"], &[]);
}

#[test]
fn test_elixir_module() {
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
    // Struct fields are flagged both in the defstruct atom list and as keys
    // in the struct literal; "Accaunt" at the module def and struct literal.
    // Lowercase "accaunt" occurrence 0 is inside "accaunts" in the
    // moduledoc; occurrence 1 is the flagged create_accaunt def.
    assert_spelling_at(
        LanguageType::Elixir,
        sample_elixir,
        &[
            ("Accaunt", &[0, 1]),
            ("handels", &[0]),
            ("accaunts", &[0]),
            ("usrrnamee", &[0, 1]),
            ("ballancee", &[0, 1]),
            ("intrest", &[0, 1]),
            ("accaunt", &[1]),
        ],
    );
}

#[test]
fn test_elixir_functions() {
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
    // Function names are flagged both at the pipeline call sites
    // (occurrence 0) and at their defp definitions (occurrence 1).
    assert_spelling_at(
        LanguageType::Elixir,
        sample_elixir,
        &[
            ("Aplies", &[0]),
            ("Performz", &[0]),
            ("Savs", &[0]),
            ("databse", &[0]),
            ("incomming", &[0]),
            ("logik", &[0]),
            ("persiste", &[0, 1]),
            ("proccess", &[0]),
            ("procesing", &[0]),
            ("ruls", &[0]),
            ("transfrom", &[0, 1]),
            ("validatte", &[0, 1]),
        ],
    );
}

#[test]
fn test_elixir_pattern_matching() {
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
    // Atoms (:succes, :failur), map keys ("conten:"), string patterns
    // ("notfication"), function definition names, call names
    // (process_notfication), and call arguments (conten occurrence 2) are
    // flagged; pattern-bound variables (resalt, reson, conten occurrence 1)
    // are not.
    assert_spelling_at(
        LanguageType::Elixir,
        sample_elixir,
        &[
            ("conten", &[0, 2]),
            ("failur", &[0]),
            ("mesage", &[0]),
            ("notfication", &[0, 1]),
            ("responce", &[0, 1]),
            ("succes", &[0]),
        ],
    );
}
