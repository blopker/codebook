use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_nix_location() {
    let sample_text = include_str!("../examples/example.nix");

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
            ("pname", &[0]),
            // Below strings are conventional / very common abbreviations in Nix, but they are not keywords,
            // so they must be flagged as misspelled
            ("nixpkgs", &[0, 2]),
            ("pkgs", &[3]),
        ],
    );
}
