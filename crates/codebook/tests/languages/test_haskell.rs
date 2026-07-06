use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_haskell_simple() {
    let sample_text = r#"
        func myArrg = do
           calculatr <- makeCowculator
           sum calculatr [ numberr1, argument2, myArrg ]
    "#;
    assert_spelling_at(
        LanguageType::Haskell,
        sample_text,
        &[
            // Haskell variables are flagged at every occurrence, not just
            // at their binding site.
            ("Arrg", &[0, 1]),
            ("calculatr", &[0, 1]),
            ("Cowculator", &[0]),
            ("numberr", &[0]),
        ],
    );
}

#[test]
fn test_haskell_string() {
    let sample_text = r#"
        let str =  "herlo, world"
        in str
    "#;
    assert_spelling(LanguageType::Haskell, sample_text, &["herlo"], &["world"]);
}

#[test]
fn test_haskell_module() {
    let sample_text = r#"
        import Data.Functoin as Func
        import Data.Function qualified as D.Funcc
    "#;
    assert_spelling(
        LanguageType::Haskell,
        sample_text,
        &["Functoin", "Funcc"],
        &[],
    );
}

#[test]
fn test_haskell_types() {
    let sample_text = r#"
        func :: forall badd . (MyTypeeClass var) => varr -> Intt
        func = varToInt
    "#;
    assert_spelling(
        LanguageType::Haskell,
        sample_text,
        // Typee is flagged at its camelCase sub-token range in MyTypeeClass.
        &["badd", "Typee", "varr", "Intt"],
        &[],
    );
}
