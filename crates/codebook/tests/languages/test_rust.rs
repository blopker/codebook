use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_rust_simple() {
    let sample_text = r#"
        fn calculat_user_age(bithDate: String) -> u32 {
            // This is an examle_function that calculates age
            let usrAge = get_curent_date() - bithDate;
            userAge
        }
    "#;
    // Identifiers are flagged at their definition (the `bithDate` parameter),
    // not at usages.
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("bith", &[0]), ("calculat", &[0]), ("examle", &[0])],
    );
}

#[test]
fn test_rust_comment_location() {
    // "mment" is also a substring of "Comment"; only the standalone word
    // (occurrence 1) is flagged.
    assert_spelling_at(
        LanguageType::Rust,
        r#"
        // Comment with a typo: mment
        "#,
        &[("mment", &[1])],
    );
}

#[test]
fn test_rust_block_comments() {
    let sample_rust = r#"
        /* Comment with a typos on multiple lines: mment

        examle
        */

        /*! Inner block doc comment: testz
        */

        /** Outer block doc comment.

        Eror.
        */
        "#;
    // "mment" is also a substring of "Comment"; only the standalone word
    // (occurrence 1) is flagged.
    assert_spelling_at(
        LanguageType::Rust,
        sample_rust,
        &[
            ("mment", &[1]),
            ("examle", &[0]),
            ("testz", &[0]),
            ("Eror", &[0]),
        ],
    );
}

#[test]
fn test_rust_struct() {
    let sample_rust = r#"
        pub struct BadSpeler {
            /// Terrible spelling: dwnloader
            pub dataz: String,
        }
        "#;
    assert_spelling(
        LanguageType::Rust,
        sample_rust,
        &["Speler", "dwnloader", "dataz"],
        &[],
    );
}

#[test]
fn test_rust_trait_impl_function_names_not_checked() {
    // https://github.com/blopker/codebook/issues/225
    // Function names in `impl Trait for Type` blocks should not be spell-checked
    // because the names are dictated by the trait, not the implementor.
    let sample_text = r#"
        struct MyType;

        impl SomeTrait for MyType {
            fn spelling_erorr(self) {
                // This comment has a typo: tset
            }
        }
    "#;
    // Comments inside the impl block are still checked.
    assert_spelling(LanguageType::Rust, sample_text, &["tset"], &["erorr"]);
}

#[test]
fn test_rust_regular_impl_function_names_checked() {
    // Regular impl blocks (not trait implementations) should still be spell-checked
    let sample_text = r#"
        struct MyType;

        impl MyType {
            fn spelling_erorr(self) {}
        }

        fn top_level_erorr() {}
    "#;
    // "erorr" is flagged in both the regular impl method and the top-level
    // function.
    assert_spelling_at(LanguageType::Rust, sample_text, &[("erorr", &[0, 1])]);
}
