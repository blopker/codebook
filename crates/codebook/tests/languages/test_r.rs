use codebook::queries::LanguageType;

use super::utils::assert_spelling;

#[test]
fn test_r_simple() {
    let sample_text = r#"
       calculatr <- function(numbr1, argumnt2=3) {
           # This is an exampl function
           numberr1 + argument2
       }
    "#;
    // Identifiers used in the function body ("numberr1") are not checked;
    // only assignments, parameters, comments, and strings are.
    assert_spelling(
        LanguageType::R,
        sample_text,
        &["argumnt", "calculatr", "exampl", "numbr"],
        &["numberr"],
    );
}

#[test]
fn test_r_string() {
    let sample_text = r#"
        my_var <- "herlo, world"
    "#;
    assert_spelling(LanguageType::R, sample_text, &["herlo"], &[]);
}

#[test]
fn test_r_kwarg() {
    let sample_text = r#"
        table2 <- dplyr::mutate(table1, mispell=nmae1 + name2, bad_spelin, olny_named_cols=3)
    "#;
    // Only keyword argument names are checked; argument values ("nmae1")
    // and positional arguments ("bad_spelin") are not.
    assert_spelling(
        LanguageType::R,
        sample_text,
        &["mispell", "olny"],
        &["nmae", "spelin"],
    );
}

#[test]
fn test_r_assign() {
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
    assert_spelling(
        LanguageType::R,
        sample_text,
        &["chiian", "leeft", "lefft", "miispell", "rihgt", "wroks"],
        &["chainn", "ingore", "lerft"],
    );
}
