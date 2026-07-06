use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_zig_simple() {
    let sample_text = r#"
const tesst = 5;
var valuue = 10;
"#;
    assert_spelling(LanguageType::Zig, sample_text, &["tesst", "valuue"], &[]);
}

#[test]
fn test_zig_strings() {
    let sample_text = r#"
test "bad speling" {
    const msg = "Hello Wolrd";
}
"#;
    assert_spelling(LanguageType::Zig, sample_text, &["speling", "Wolrd"], &[]);
}

#[test]
fn test_zig_functions() {
    let sample_text = r#"
fn addNumberrs(firstt: i32, seconnd: i32) i32 {
    return firstt + seconnd;
}
"#;
    assert_spelling_at(
        LanguageType::Zig,
        sample_text,
        &[
            // Flagged at its camelCase sub-token range inside addNumberrs.
            ("Numberrs", &[0]),
            // Parameters are flagged at their declaration, not at usages
            // in the function body.
            ("firstt", &[0]),
            ("seconnd", &[0]),
        ],
    );
}
