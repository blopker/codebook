use codebook::queries::LanguageType;

use super::utils::{
    assert_spelling, assert_spelling_at, assert_spelling_at_with, get_processor_with_tags,
};

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
fn test_rust_union() {
    let sample_text = r#"
        union EentOrFlote {
            eent: i32,
            flote: f32,
        }

        fn main() {
            let u = EentOrFlote { flote: 1.0 };
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[
            ("Eent", &[0]),
            ("Flote", &[0]),
            ("eent", &[0]),
            ("flote", &[0]),
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

#[test]
fn test_rust_module_name() {
    let sample_text = r#"
        pub mod mawde;

        mod modyeule {
            pub fn f() {}
        }

        fn main() {
            modyeule::f();
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("mawde", &[0]), ("modyeule", &[0])],
    );
}

#[test]
fn test_rust_use_declaration_and_renaming() {
    // Only user-defined renames should be checked, and only where they are defined.
    let sample_text = r#"
        use some_crate::badlynamed;
        use some_crate::otherbad as mispeling;
        use other_crate::{scoopeduse, Skoped as Skopedbad};

        fn main() {
            mispeling();
            let x = Skopedbad::new();
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("mispeling", &[0]), ("Skopedbad", &[0])],
    );
}

#[test]
fn test_rust_extern_crate_and_renaming() {
    // Only user-defined renames should be checked, and only where they are defined.
    let sample_text = r#"
        extern crate horiblspeler;
        extern crate relybad as kanotspel;

        fn main() {
            kanotspel::f();
        }
    "#;
    assert_spelling_at(LanguageType::Rust, sample_text, &[("kanotspel", &[0])]);
}

#[test]
fn test_rust_pattern_positions() {
    // Bindings should be checked where they are bound in every position a
    // pattern can appear, and tagged as parameters if in function or closure
    // parameters. Uppercase names in refutable patterns may be constants or
    // unit variants, like `MAAAX`, which should only be checked where defined.
    let sample_text = r#"
        const MAAAX: i32 = 1234;

        fn f(paramwun: i32) {
            let c = |klozplain, kloztype: i32, mut klozmut| klozplain + kloztype + klozmut;
            let letwun = 1;
            let Some(letelse) = opt else { return };
            if let Some(iflet) = opt {}
            while let Some(whylet) = it.next() {}
            for indeks in v {}
            match s {
                armwun => {}
                MAAAX => {}
            }
        }
    "#;
    assert_spelling_at_with(
        &get_processor_with_tags(vec!["identifier.parameter"], vec![]),
        LanguageType::Rust,
        sample_text,
        &[
            ("paramwun", &[0]),
            ("klozplain", &[0]),
            ("kloztype", &[0]),
            ("klozmut", &[0]),
        ],
    );
    assert_spelling_at_with(
        &get_processor_with_tags(vec!["identifier.variable"], vec![]),
        LanguageType::Rust,
        sample_text,
        &[
            ("letwun", &[0]),
            ("letelse", &[0]),
            ("iflet", &[0]),
            ("whylet", &[0]),
            ("indeks", &[0]),
            ("armwun", &[0]),
        ],
    );
    assert_spelling_at_with(
        &get_processor_with_tags(vec![], vec!["identifier.parameter", "identifier.variable"]),
        LanguageType::Rust,
        sample_text,
        &[("MAAAX", &[0])],
    );
}

#[test]
fn test_rust_pattern_kinds() {
    // Bindings nested to arbitrary depth in each kind of pattern should be
    // checked, and tagged as parameters if in function or closure parameters.
    // Struct names and field names, including shorthand like `Poynt { wyy }`,
    // are defined elsewhere and should not be checked. `rappur` is lowercase so
    // the lowercase filter alone can't hide it.
    let sample_text = r#"
        struct Poynt { exx: i32, wyy: i32 }
        struct rappur(i32);

        fn f(
            (tupparam, _): (i32, i32),
            [slyceparam, ..]: [i32; 2],
            rappur(rapparam): rappur,
            Poynt { exx: feeldparam, wyy }: Poynt,
            ref refparam: i32,
            (mut mutparam, _): (i32, i32),
            &amparam: &i32,
            kapparam @ _: i32,
            (_, [rappur(deepparam), ..]): (i32, [rappur; 1]),
        ) {
            let (tuplet, _) = pair;
            let [slycelet, ..] = arr;
            let rappur(raplet) = w;
            let Poynt { exx: feeldlet, wyy } = p;
            let ref reflet = x;
            let (mut mutlet, _) = pair;
            let &amplet = r;
            let kaplet @ (_, _) = pair;
            let (_, [rappur(deeplet), ..]) = nested;
        }
    "#;
    assert_spelling_at_with(
        &get_processor_with_tags(vec!["identifier.parameter"], vec![]),
        LanguageType::Rust,
        sample_text,
        &[
            ("tupparam", &[0]),
            ("slyceparam", &[0]),
            ("rapparam", &[0]),
            ("feeldparam", &[0]),
            ("refparam", &[0]),
            ("mutparam", &[0]),
            ("amparam", &[0]),
            ("kapparam", &[0]),
            ("deepparam", &[0]),
        ],
    );
    assert_spelling_at_with(
        &get_processor_with_tags(vec!["identifier.variable"], vec![]),
        LanguageType::Rust,
        sample_text,
        &[
            ("tuplet", &[0]),
            ("slycelet", &[0]),
            ("raplet", &[0]),
            ("feeldlet", &[0]),
            ("reflet", &[0]),
            ("mutlet", &[0]),
            ("amplet", &[0]),
            ("kaplet", &[0]),
            ("deeplet", &[0]),
        ],
    );
    // Other misspellings should only be the definitions of the structs and
    // fields
    assert_spelling_at_with(
        &get_processor_with_tags(vec![], vec!["identifier.parameter", "identifier.variable"]),
        LanguageType::Rust,
        sample_text,
        &[
            ("Poynt", &[0]),
            ("exx", &[0]),
            ("wyy", &[0]),
            ("rappur", &[0]),
        ],
    );
}

#[test]
fn test_rust_unidiomatic_uppercase_bindings() {
    // Uppercase locals and parameters are unidiomatic Rust, but should still
    // be checked where they must be bindings. This includes anywhere in
    // parameters, top-level `let` and `for` patterns, after `ref` or `mut`,
    // and before `@`. Elsewhere in patterns, they may be constants or unit
    // variants, so they should not be checked, like `KONSTANT` and `Grean` in
    // the match arms. `Klozwun` is both a `mut` pattern and inside a closure
    // parameter, so it must be checked only once, as a parameter.
    let sample_text = r#"
        const KONSTANT: i32 = 1;
        enum Kolor { Grean }
        use Kolor::*;

        fn f(Paramwun: i32, Paramkap @ _: i32) {
            let LETWUN = 1;
            let c = |mut Klozwun| Klozwun + 1;
            for Indeks in v {}
            if let Some(ref REFWUN) = opt {}
            match s {
                Some(mut Mutwun) => {}
                KAPCHUR @ 0..=9 => {}
                Some(KONSTANT) => {}
                Grean => {}
            }
        }
    "#;
    assert_spelling_at_with(
        &get_processor_with_tags(vec!["identifier.parameter"], vec![]),
        LanguageType::Rust,
        sample_text,
        &[("Paramwun", &[0]), ("Klozwun", &[0]), ("Paramkap", &[0])],
    );
    assert_spelling_at_with(
        &get_processor_with_tags(vec!["identifier.variable"], vec![]),
        LanguageType::Rust,
        sample_text,
        &[
            ("LETWUN", &[0]),
            ("Indeks", &[0]),
            ("REFWUN", &[0]),
            ("Mutwun", &[0]),
            ("KAPCHUR", &[0]),
        ],
    );
    // Other misspellings should only be the definitions of the constant, enum,
    // and variant
    assert_spelling_at_with(
        &get_processor_with_tags(vec![], vec!["identifier.parameter", "identifier.variable"]),
        LanguageType::Rust,
        sample_text,
        &[("KONSTANT", &[0]), ("Kolor", &[0]), ("Grean", &[0])],
    );
}

#[test]
fn test_rust_generic_parameters() {
    // Type, const, and lifetime parameters should be checked where they are
    // declared, including higher-ranked `for<'a>` lifetimes, but not where
    // they are used.
    let sample_text = r#"
        fn f<'lyfe, Tayp: Clone, const SAIZE: usize>(x: &'lyfe [Tayp; SAIZE])
        where
            Tayp: for<'haiyur> Fn(&'haiyur str),
        {
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[
            ("lyfe", &[0]),
            ("Tayp", &[0]),
            ("SAIZE", &[0]),
            ("haiyur", &[0]),
        ],
    );
}

#[test]
fn test_rust_labels() {
    // Loop and block labels should be checked where they are declared, but
    // not in `break` or `continue`.
    let sample_text = r#"
        fn labels() {
            'eenfeeneet: loop { break 'eenfeeneet; }
            'whyle: while x { continue 'whyle; }
            'fohr: for i in v { continue 'fohr; }
            let y = 'bluck: { break 'bluck 1; };
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[
            ("eenfeeneet", &[0]),
            ("whyle", &[0]),
            ("fohr", &[0]),
            ("bluck", &[0]),
        ],
    );
}

#[test]
fn test_rust_macro_rules() {
    // A `macro_rules!` name and its metavariables should be checked where they
    // are declared, but not where the macro is invoked or the metavariables are
    // used. Each arm declares its own metavariables, so a name repeated across
    // arms should be checked in each.
    let sample_text = r#"
        macro_rules! make_payre {
            ($furst:expr, $($restt:literal),*) => {
                ($furst, [$($restt),*])
            };
            ($furst:expr) => {
                ($furst, [])
            };
        }

        fn main() {
            let pair1 = make_payre!(1, 2, 3);
            let pair2: (i32, [i32; 0]) = make_payre!(4);
        }
    "#;
    assert_spelling_at(
        LanguageType::Rust,
        sample_text,
        &[("payre", &[0]), ("furst", &[0, 2]), ("restt", &[0])],
    );
}
