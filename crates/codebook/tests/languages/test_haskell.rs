use codebook::queries::LanguageType;

#[test]
fn test_haskell_simple() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
        func myArrg = do
           calculatr <- makeCowculator
           sum calculatr [ numberr1, argument2, myArrg ]
    "#;
    let expected = vec!["Arrg", "Cowculator", "calculatr", "numberr"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Haskell), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}

#[test]
fn test_haskell_string() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
        let str =  "herlo, world"
        in str
    "#;
    let expected = vec!["herlo"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Haskell), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}

#[test]
fn test_haskell_module() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
        import Data.Functoin as Func
        import Data.Function qualified as D.Funcc
    "#;
    let expected = vec!["Funcc", "Functoin"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Haskell), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}

#[test]
fn test_haskell_types() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"
        func :: forall badd . (MyTypeeClass var) => varr -> Intt
        func = varToInt
    "#;
    let expected = vec!["Intt", "Typee", "badd", "varr"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Haskell), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}
