use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_css_location() {
    let sample_css = r#"
        .test {
            color: red;
        }
        .testz {
            color: blue;
        }
"#;
    assert_spelling(LanguageType::Css, sample_css, &["testz"], &[]);
}

#[test]
fn test_css_escape_sequences_separate_words() {
    let sample_text = r#".a::before { content: "this\9 is\A indented wrold"; }"#;
    assert_spelling_at(LanguageType::Css, sample_text, &[("wrold", &[0])]);
}
