use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_latex_comments() {
    let sample_text = r#"
% This is a coment with a typo
% Another commnet with wrng spelling
\documentclass{article}
    "#;
    // Command names (\documentclass) and their arguments are not checked.
    assert_spelling(
        LanguageType::Latex,
        sample_text,
        &["coment", "commnet", "wrng"],
        &["documentclass", "article"],
    );
}

#[test]
fn test_latex_text_content() {
    let sample_text = r#"
\section{Introducton}

This is an exampl of text with speling errors.
    "#;
    assert_spelling(
        LanguageType::Latex,
        sample_text,
        &["Introducton", "exampl", "speling"],
        &[],
    );
}

#[test]
fn test_latex_sections_and_text() {
    let sample_text = r#"
\section{Methology}

The methology section describs the approach.

\subsection{Bakground}

In this secion we discuss importnt concepts.
    "#;
    assert_spelling(
        LanguageType::Latex,
        sample_text,
        &[
            "Bakground",
            "Methology",
            "describs",
            "importnt",
            "methology",
            "secion",
        ],
        &[],
    );
}

#[test]
fn test_latex_itemize() {
    let sample_text = r#"
\begin{itemize}
    \item First itm with algoritm
    \item Second itm about formulas
\end{itemize}
    "#;
    // "itm" appears in both \item lines and is flagged at both; occurrence 1
    // is the substring inside "algoritm", which is flagged as its own word.
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[("algoritm", &[0]), ("itm", &[0, 2])],
    );
}

#[test]
fn test_latex_mixed_content() {
    let sample_text = r#"
% Comment: calcuate the result
\section{Resuts}

The resuts show our aproach is efective.

\begin{equation}
    E = mc^2 \label{eq:enrgy}
\end{equation}

As shown in Equation~\ref{eq:enrgy}, the relatioship is clear.
    "#;
    // Exact set equality: command names (equation, label, ref, begin, end,
    // section) can't appear in the flagged set. Label names ARE checked:
    // "enrgy" is flagged both in \label and \ref.
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[
            ("Resuts", &[0]),
            ("aproach", &[0]),
            ("calcuate", &[0]),
            ("efective", &[0]),
            ("enrgy", &[0, 1]),
            ("relatioship", &[0]),
            ("resuts", &[0]),
        ],
    );
}

#[test]
fn test_latex_comprehensive() {
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
    // Exact set equality: command names (documentclass, article, title,
    // begin, end, document, section, subsection) can't appear in the flagged
    // set. "spel" also occurs inside "speling" (occurrence 0); only the
    // standalone word is flagged as "spel".
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[
            ("Analyss", &[0]),
            ("Introducton", &[0]),
            ("Sampel", &[0]),
            ("analyss", &[0]),
            ("coment", &[0]),
            ("docment", &[0]),
            ("paterns", &[0]),
            ("spel", &[1]),
            ("speling", &[0]),
            ("wrng", &[0]),
        ],
    );
}
