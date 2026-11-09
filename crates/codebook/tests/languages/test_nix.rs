use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_nix_location() {
    let sample_text = r#"
# Commment with a typo
{
  description = "A useful dev shell for the packge";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flakeUtils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flakeUtils }:
    flakeUtils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        verion = "0.1.0";
      in
      {
        devShells.defualt = pkgs.mkShell {
          buildInputs = [ pkgs.cargo pkgs.rustc ];
          shellHook = ''
            echo "Entring dev shell version ${verion}"
          '';
        };
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "my-projet";
          version = verion;
          src = ./.;
        };
      });
}
"#;

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
fn test_nix_extension_detects_language() {
    use codebook::queries::get_language_name_from_filename;
    assert_eq!(
        get_language_name_from_filename("flake.nix"),
        LanguageType::Nix
    );
    assert_eq!(
        get_language_name_from_filename("/etc/nixos/configuration.nix"),
        LanguageType::Nix
    );
}
