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
fn test_rust_enum() {
    let sample_text = r#"
        enum Eeneum {
            Yoonit,
            Tuppel(i32),
            Struckte { feyeld: i32 },
        }

        fn main() {
            let x = Eeneum::Yoonit;
            let y = Eeneum::Tuppel(42);
            let z = Eeneum::Struckte { feyeld: 42 };
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[
            ("Eeneum", &[0]),
            ("Yoonit", &[0]),
            ("Tuppel", &[0]),
            ("Struckte", &[0]),
            ("feyeld", &[0]),
        ],
    );
}

#[test]
fn test_rust_type_alias() {
    let sample_text = r#"
        type MaybeYoosize = Option<usize>;

        fn main() {
            type Rezoult = Result<usize, &'static str>;

            let a: MaybeYoosize = None;
            let b = |x: Rezoult| -> MaybeYoosize { x.ok() };
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("Yoosize", &[0]), ("Rezoult", &[0])],
    );
}

#[test]
fn test_rust_trait_names_types_functions() {
    // Traits' names, associated types, functions without default bodies
    // (`function_signature_item`), and functions with default bodies
    // (`function_item`) should all be checked where defined and not where
    // implemented or used.
    let sample_text = r#"
        trait Trayet {
            type Yaytem;
            fn meethode(&self);
            fn foncshun() {
                println!("hello");
            }
        }

        impl Trayet for i32 {
            type Yaytem = u32;
            fn meethode(&self) {
                println!("I'm an i32: {self}");
            }
            fn foncshun() {
                println!("hello from i32");
            }
        }

        fn f<T: Trayet>(x: T, y: T::Yaytem) {
            x.meethode();
            T::foncshun();
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[
            ("Trayet", &[0]),
            ("Yaytem", &[0]),
            ("meethode", &[0]),
            ("foncshun", &[0]),
        ],
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

#[test]
fn test_rust_escape_sequences_separate_words() {
    let sample_text = r#"fn main() { println!("this\nis\tindented\u{1F600}wrold"); }"#;
    assert_spelling_at(LanguageType::Rust, sample_text, &[("wrold", &[0])]);
}

#[test]
fn test_rust_const_and_static_in_module_and_fn_body() {
    let sample_text = r#"
        const KYANST: i32 = 42;
        static STYATIK: i32 = 42;

        const X: i32 = KYANST + STYATIK;

        fn main() {
            const KYANST: i32 = 42;
            static STYATIK: i32 = 42;

            let x = KYANST + STYATIK;
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("KYANST", &[0, 2]), ("STYATIK", &[0, 2])],
    );
}

#[test]
fn test_rust_associated_consts_checked_where_defined() {
    // Associated consts should be checked where they're defined, in trait
    // definitions and inherent impls, but not in trait impls, where the trait
    // chooses the name (like the function names from issue #225). However,
    // values in trait impls should still be checked.
    let sample_text = r#"
        trait Shape {
            const SYDEZ: u32;
            const LABEL: &'static str;
        }

        struct Square;

        impl Shape for Square {
            const SYDEZ: u32 = 4;
            const LABEL: &'static str = "four kornerz";
        }

        impl Square {
            const DYAGONALLS: u32 = 2;
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("SYDEZ", &[0]), ("kornerz", &[0]), ("DYAGONALLS", &[0])],
    );
}

#[test]
fn test_rust_extern_items_not_checked() {
    // Items in extern blocks are named by the foreign library.
    let sample_text = r#"
        unsafe extern "C" {
            static ernoh: i32;
            fn strnkpi(dst: *mut u8, src: *const u8, dsize: usize);
        }

        static STYATIK: i32 = 0;
    "#;
    assert_spelling(
        LanguageType::Rust,
        sample_text,
        &["STYATIK"],
        &["ernoh", "strnkpi"],
    );
}
