use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_c_simple() {
    let sample_text = r#"
        int calculatr(int numbr1, int numbr2, char operashun) {
            // This is an exampl function that performz calculashuns
            int resalt = 0;
            int misspellled;
            return resalt + misspellled;
        }
    "#;
    assert_spelling_at(
        LanguageType::C,
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
            // Variables are flagged at their declaration, not at usages.
            ("resalt", &[0]),
            ("misspellled", &[0]),
        ],
    );
}

#[test]
fn test_c_comment_location() {
    let sample_c = r#"
        // Structur definition with misspellings
    "#;
    assert_spelling(LanguageType::C, sample_c, &["Structur"], &[]);
}

#[test]
fn test_c_struct() {
    let sample_c = r#"
        struct UserAccaunt {
            char* usrrnamee;
            int ballancee;
            float intrest_rate;
        };
    "#;
    assert_spelling(
        LanguageType::C,
        sample_c,
        // `Accaunt` and `intrest` are flagged at sub-token ranges inside
        // `UserAccaunt` and `intrest_rate`.
        &["Accaunt", "usrrnamee", "ballancee", "intrest"],
        &["User", "rate"],
    );
}

#[test]
fn test_c_macros() {
    let sample_text = r#"
        #define MACROCONST 3
        #define MACROFUNC(macroparam) macroparam + 1
    "#;
    assert_spelling_at(
        LanguageType::C,
        sample_text,
        &[
            ("MACROCONST", &[0]),
            ("MACROFUNC", &[0]),
            // Flagged at the parameter declaration, not the macro body usage.
            ("macroparam", &[0]),
        ],
    );
}

#[test]
fn test_c_unions() {
    let sample_text = r#"union myunion { int int_val; };"#;
    assert_spelling(LanguageType::C, sample_text, &["myunion"], &["int_val"]);
}

#[test]
fn test_c_variable_declarations() {
    let sample_text = r#"
        int arrayy[3];
        int* pointerr;
        int* pointerrarray[3];
        enum Role rolee;
        union Union unionn;
        struct User userr;"#;
    assert_spelling_at(
        LanguageType::C,
        sample_text,
        &[
            ("arrayy", &[0]),
            // Occurrence 1 is the substring inside `pointerrarray`, which is
            // flagged as its own whole word instead.
            ("pointerr", &[0]),
            ("pointerrarray", &[0]),
            ("rolee", &[0]),
            ("unionn", &[0]),
            ("userr", &[0]),
        ],
    );
}

#[test]
fn test_c_variable_initializers() {
    // Note: variables with initializers have slightly different syntax tree
    // representations, so it useful to test them along with plain declarations.
    let sample_text = r#"
        int arrayy[3] = {};
        int* pointerr = NULL;
        int* pointerrarray[3] = {};
        enum Role rolee = ROLE1;
        union Union unionn = 10;
        struct User userr = {};"#;
    assert_spelling_at(
        LanguageType::C,
        sample_text,
        &[
            ("arrayy", &[0]),
            // Occurrence 1 is the substring inside `pointerrarray`.
            ("pointerr", &[0]),
            ("pointerrarray", &[0]),
            ("rolee", &[0]),
            ("unionn", &[0]),
            ("userr", &[0]),
        ],
    );
}

#[test]
fn test_c_field_declarations() {
    let sample_text = r#"
        struct MyStruct {
            int arrayy[3];
            int* pointerr;
            int* pointerrarray[3];
            enum Role rolee;
            union Union unionn;
            struct User userr;
        }"#;
    assert_spelling_at(
        LanguageType::C,
        sample_text,
        &[
            ("arrayy", &[0]),
            // Occurrence 1 is the substring inside `pointerrarray`.
            ("pointerr", &[0]),
            ("pointerrarray", &[0]),
            ("rolee", &[0]),
            ("unionn", &[0]),
            ("userr", &[0]),
        ],
    );
}

#[test]
fn test_c_strings() {
    // Note: we do not do spell checking across string concatenations,
    // just individual strings.
    let sample_text = r#"
        char* str1 = "aaaa bbbb";
        str1 = "cccc" "valid string" "dddd";
        printf("I'm a multiline stringg\n"
               "withh\nyyy");
    "#;
    assert_spelling(
        LanguageType::C,
        sample_text,
        &["aaaa", "bbbb", "cccc", "dddd", "stringg", "withh", "yyy"],
        &["valid"],
    );
}

#[test]
fn test_c_typedef() {
    let sample_text = r#"typedef int Mispelll;"#;
    assert_spelling(LanguageType::C, sample_text, &["Mispelll"], &[]);
}

#[test]
fn test_c_enum() {
    let sample_text = r#"enum Colrs { Grean };"#;
    assert_spelling(LanguageType::C, sample_text, &["Colrs", "Grean"], &[]);
}

#[test]
fn test_c_type_uses() {
    let sample_text = r#"
        enum Colorr color;
        union Dataa data;
        struct Userr user;
    "#;
    // Type names at use sites are not spell-checked (only definitions are).
    assert_spelling(
        LanguageType::C,
        sample_text,
        &[],
        &["Colorr", "Dataa", "Userr"],
    );
}
