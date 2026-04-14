use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};


#[test]
fn test_ocaml_location() {
    super::utils::init_logging();
    let sample_text = r#"
(* Commment with a typo *)
type usr_recrod = {
  name: string;
  agge: int;
}
let greet_usr (persn : usr_recrod) =
  let msega = "Helllo " ^ persn.name in
  print_endline msega
let () =
  let u = { name = "Alise"; agge = 30 } in
  greet_usr u
"#;

    let expected = vec![
        WordLocation::new(
            "Commment".to_string(),
            vec![TextRange {
                start_byte: 4,
                end_byte: 12,
            }],
        ),
        WordLocation::new(
            "recrod".to_string(),
            vec![
                // type definition
                TextRange {
                    start_byte: 37,
                    end_byte: 43,
                },
                // parameter annotation in greet_usr
                TextRange {
                    start_byte: 106,
                    end_byte: 112,
                },
            ],
        ),
        WordLocation::new(
            "agge".to_string(),
            vec![
                // field definition
                TextRange {
                    start_byte: 66,
                    end_byte: 70,
                },
                // field assignment in let u = { ... }
                TextRange {
                    start_byte: 215,
                    end_byte: 219,
                },
            ],
        ),
        WordLocation::new(
            "persn".to_string(),
            vec![TextRange {
                start_byte: 94,
                end_byte: 99,
            }],
        ),
        WordLocation::new(
            "msega".to_string(),
            vec![TextRange {
                start_byte: 122,
                end_byte: 127,
            }],
        ),
        WordLocation::new(
            "Helllo".to_string(),
            vec![TextRange {
                start_byte: 131,
                end_byte: 137,
            }],
        ),
        WordLocation::new(
            "Alise".to_string(),
            vec![TextRange {
                start_byte: 207,
                end_byte: 212,
            }],
        ),
    ];

    let not_expected = [
        "and",
        "as",
        "assert",
        "begin",
        "class",
        "constraint",
        "do",
        "done",
        "downto",
        "effect",
        "else",
        "end",
        "exception",
        "external",
        "for",
        "fun",
        "function",
        "functor",
        "if",
        "in",
        "include",
        "inherit",
        "initializer",
        "lazy",
        "let",
        "match",
        "method",
        "module",
        "mutable",
        "new",
        "nonrec",
        "object",
        "of",
        "open",
        "private",
        "rec",
        "sig",
        "struct",
        "then",
        "to",
        "try",
        "type",
        "val",
        "virtual",
        "when",
        "while",
        "with",
    ];

    let processor = super::utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::OCaml), None)
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
