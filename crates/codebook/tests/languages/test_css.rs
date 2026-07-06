use codebook::queries::LanguageType;

use super::utils::assert_spelling;

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
