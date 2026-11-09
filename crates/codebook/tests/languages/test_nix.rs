use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_nix_location() {
    let sample_text = include_str!("../../../../examples/example.nix");

    assert_spelling_at(
        LanguageType::Nix,
        sample_text,
        &[
            ("Commment", &[0]),
            ("packge", &[0]),
            // Flagged at the let binding declaration only;
            // the `${verion}` interpolation and `version = verion;` usages are not flagged
            ("verion", &[0]),
            ("defualt", &[0]),
            ("Entring", &[0]),
            ("projet", &[0]),
        ],
    );
}

#[test]
fn test_nix_nixpkgs_vocabulary_is_allowed() {
    // `nixpkgs`, `pkgs`, `pname` and friends are nixpkgs vocabulary rather than
    // user-chosen names, so they live in word_lists/nix.txt. Flagging them in
    // every flake would be noise the user cannot act on.
    let sample_text = r#"
{ pkgs, nixpkgs }:
let
  pname = "my-app";
in
pkgs.stdenv.mkDerivation {
  inherit pname;
  version = "0.1.0";
  shellHook = "echo hello from ${pname}";
}
"#;
    assert_spelling(
        LanguageType::Nix,
        sample_text,
        &[],
        &["nixpkgs", "pkgs", "pname", "stdenv"],
    );
}

#[test]
fn test_nix_attrpath_checks_only_identifier_keys() {
    // Quoted keys are covered by the string capture, and `${...}` keys are
    // expressions, so only the bare identifier parts of a binding's attrpath
    // are definitions to check.
    let sample_text = r#"
{ cfg, ... }:
{
  boot.kernel.sysctl."net.core.buffer_maxx" = 1;
  users.${cfg.usre}.home = "/home";
  servcies.foo.enable = true;
}
"#;
    assert_spelling_at(
        LanguageType::Nix,
        sample_text,
        &[("maxx", &[0]), ("servcies", &[0])],
    );
}

#[test]
fn test_nix_example_file_detects_language() {
    // Goes through `spell_check_file` so the `.nix` extension is what picks
    // the grammar, the same way the language server resolves an open buffer.
    let processor = super::utils::get_processor();
    let results = processor
        .spell_check_file("../../examples/example.nix")
        .unwrap();
    let mut misspelled: Vec<&str> = results.iter().map(|r| r.word.as_str()).collect();
    misspelled.sort_unstable();
    assert_eq!(
        misspelled,
        [
            "Commment", "Entring", "defualt", "packge", "projet", "verion"
        ]
    );
}
