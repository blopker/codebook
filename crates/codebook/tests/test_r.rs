use codebook::queries::LanguageType;
mod utils;

#[test]
fn test_r_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
       calculatr <- function(numbr1, argumnt2=3) {
           # This is an exampl function
           numberr1 + argument2
       }
    "#;
    let expected = vec!["argumnt", "calculatr", "exampl", "numbr"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::R), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}

#[test]
fn test_r_string() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        my_var <- "herlo, world"
    "#;
    let expected = vec!["herlo"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::R), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}

#[test]
fn test_r_kwarg() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        table2 <- dplyr::mutate(table1, mispell=nmae1 + name2, bad_spelin, olny_named_cols=3)
    "#;
    let expected = vec!["mispell", "olny"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::R), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}

#[test]
fn test_r_assign() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"
        list$miispell = list()
        list$mispell$chiian <- 1
        list$mispell$chainn[1:3] <- 2 # Should not get checked
        list$ingore@atsigns = 3
        4 -> right@atsigns$wroks
        lerft -> rihgt # Only right-side gets checked
        leeft <- 3
        lefft = 2
    "#;
    let expected = vec!["chiian", "leeft", "lefft", "miispell", "rihgt", "wroks"];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::R), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    assert_eq!(misspelled, expected);
}
