use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_ocaml_location() {
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
    // OCaml keywords are not flagged; exact set equality guards that.
    assert_spelling_at(
        LanguageType::OCaml,
        sample_text,
        &[
            ("Commment", &[0]),
            // Flagged at the type definition and at the parameter type
            // annotation (both captured), via snake_case split of usr_recrod.
            ("recrod", &[0, 1]),
            // Flagged at the field definition and at the record-literal
            // assignment in `let u = { ... }`.
            ("agge", &[0, 1]),
            // Flagged at the parameter definition; the `persn.name` usage
            // is not.
            ("persn", &[0]),
            // Flagged at the let binding; the `print_endline msega` usage
            // is not.
            ("msega", &[0]),
            ("Helllo", &[0]),
            ("Alise", &[0]),
        ],
    );
}

#[test]
fn test_ocaml_format_strings_and_quoted_strings_captured_once() {
    // Format directives (`%d`, `@[`) aren't words, and a quoted string's
    // content is captured once.
    let sample_text = r#"let () = Printf.printf "%d itemms %s\n" 3 "x"
let s = {id|some quoted textt|id}
"#;
    assert_spelling_at(
        LanguageType::OCaml,
        sample_text,
        &[("itemms", &[0]), ("textt", &[0])],
    );
}

#[test]
fn test_ocaml_language_ids() {
    use std::str::FromStr;

    // VS Code sends "ocaml.interface" for .mli files
    for id in ["ocaml", "ocaml.interface"] {
        assert_eq!(LanguageType::from_str(id), Ok(LanguageType::OCaml), "{id}");
    }
}
