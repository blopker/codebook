use codebook::queries::LanguageType;

use super::utils::assert_spelling;

#[test]
fn test_toml_location() {
    let sample_toml = r#"
        name = "testx"
        [dependencies]
        toml = "0.5.8"
        testz = "0.1.0"
"#;
    // Dependency keys ("testz") are not spell-checked; string values are.
    assert_spelling(LanguageType::TOML, sample_toml, &["testx"], &["testz"]);
}
