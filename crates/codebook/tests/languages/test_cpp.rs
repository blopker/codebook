use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_cpp_simple() {
    let sample_text = r#"
        int calculatr(int numbr1, int numbr2, char operashun) {
            // This is an exampl function that performz calculashuns
            int resalt = 0;
            int misspellled;
            misspellled = 20;
            return resalt + misspellled;
        }
    "#;
    assert_spelling_at(
        LanguageType::Cpp,
        sample_text,
        &[
            ("calculatr", &[0]),
            // Both parameter declarations are flagged at the `numbr`
            // sub-token range.
            ("numbr", &[0, 1]),
            ("operashun", &[0]),
            ("exampl", &[0]),
            ("performz", &[0]),
            ("calculashuns", &[0]),
            // Variables are flagged at their declaration, not at the
            // assignment or return usages.
            ("resalt", &[0]),
            ("misspellled", &[0]),
        ],
    );
}

#[test]
fn test_cpp_comment_location() {
    let sample_cpp = r#"
        // Structur definition with misspellings
    "#;
    assert_spelling(LanguageType::Cpp, sample_cpp, &["Structur"], &[]);
}

#[test]
fn test_cpp_class() {
    let sample_cpp = r#"
        class UserAccaunt {
            std::string usrrnamee;
            int ballancee;
            float intrest_rate;
        };
    "#;
    assert_spelling(
        LanguageType::Cpp,
        sample_cpp,
        // `Accaunt` and `intrest` are flagged at sub-token ranges inside
        // `UserAccaunt` and `intrest_rate`.
        &["Accaunt", "usrrnamee", "ballancee", "intrest"],
        &["User", "rate"],
    );
}

#[test]
fn test_cpp_multiline_string_concat() {
    let sample_text = r#"
        const char* message =
            "This is a verry long string\n"
            "that continuez on multiple linez\n"
            "with lots of speling misstakes\n";
    "#;
    assert_spelling(
        LanguageType::Cpp,
        sample_text,
        &["continuez", "linez", "misstakes", "speling", "verry"],
        &["message"],
    );
}

#[test]
fn test_cpp_vector_string_literals() {
    let sample_text = r#"
        std::vector<std::string> mesages = { "Helo", "Wrold", "Cpp" };
    "#;
    assert_spelling(
        LanguageType::Cpp,
        sample_text,
        &["Helo", "Wrold", "mesages"],
        &["Cpp"],
    );
}

#[test]
fn test_cpp_stream_string_literal() {
    let sample_text = r#"
        std::cout << "Currect anser!\n youu best";
    "#;
    assert_spelling(
        LanguageType::Cpp,
        sample_text,
        &["Currect", "anser", "youu"],
        &["best"],
    );
}
