use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_latex_comments() {
    utils::init_logging();
    let sample_text = r#"
% This is a coment with a typo
% Another commnet with wrng spelling
\documentclass{article}
    "#;
    let expected = vec![
        WordLocation::new(
            "coment".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "commnet".to_string(),
            vec![TextRange {
                start_byte: 42,
                end_byte: 49,
            }],
        ),
        WordLocation::new(
            "wrng".to_string(),
            vec![TextRange {
                start_byte: 55,
                end_byte: 59,
            }],
        ),
    ];
    let not_expected = vec!["documentclass", "article"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Latex), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled
            .iter()
            .find(|r| r.word == e.word)
            .unwrap_or_else(|| panic!("Word '{}' not found in misspelled list", e.word));
        assert_eq!(miss.locations, e.locations);
    }
    for word in not_expected {
        assert!(!misspelled.iter().any(|r| r.word == word));
    }
}

#[test]
fn test_latex_text_content() {
    utils::init_logging();
    let sample_text = r#"
\section{Introducton}

This is an exampl of text with speling errors.
    "#;
    let expected = vec!["Introducton", "exampl", "speling"];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Latex), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_latex_sections_and_text() {
    utils::init_logging();
    let sample_text = r#"
\section{Methology}

The methology section describs the approach.

\subsection{Bakground}

In this secion we discuss importnt concepts.
    "#;
    let expected = vec![
        "Bakground",
        "Methology",
        "describs",
        "importnt",
        "methology",
        "secion",
    ];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Latex), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_latex_itemize() {
    utils::init_logging();
    let sample_text = r#"
\begin{itemize}
    \item First itm with algoritm
    \item Second itm about formulas
\end{itemize}
    "#;
    let expected = vec!["algoritm", "itm"];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Latex), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_latex_mixed_content() {
    utils::init_logging();
    let sample_text = r#"
% Comment: calcuate the result
\section{Resuts}

The resuts show our aproach is efective.

\begin{equation}
    E = mc^2 \label{eq:enrgy}
\end{equation}

As shown in Equation~\ref{eq:enrgy}, the relatioship is clear.
    "#;
    let expected = vec![
        "Resuts",
        "aproach",
        "calcuate",
        "efective",
        "enrgy",
        "relatioship",
        "resuts",
    ];
    let not_expected = vec!["equation", "label", "ref", "begin", "end", "section"];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Latex), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    for word in not_expected {
        assert!(!misspelled.contains(&word));
    }
}

#[test]
fn test_latex_comprehensive() {
    utils::init_logging();
    let sample_text = r#"
\documentclass{article}

% This coment has typos: wrng and speling
\title{A Sampel Document}

\begin{document}

\section{Introducton}

This docment demonstrates the spel checker.

\subsection{Analyss}

The analyss reveals paterns in the data.

\end{document}
    "#;
    let expected = vec![
        "Analyss",
        "Introducton",
        "Sampel",
        "analyss",
        "coment",
        "docment",
        "paterns",
        "spel",
        "speling",
        "wrng",
    ];
    let not_expected = vec![
        "documentclass",
        "article",
        "title",
        "begin",
        "end",
        "document",
        "section",
        "subsection",
    ];
    let processor = utils::get_processor();
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Latex), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
    for word in not_expected {
        assert!(!misspelled.contains(&word));
    }
}
