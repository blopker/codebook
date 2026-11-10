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
    // section) can't appear in the flagged set. Label names are checked
    // where they're defined: "enrgy" is flagged in \label, not in \ref.
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[
            ("Resuts", &[0]),
            ("aproach", &[0]),
            ("calcuate", &[0]),
            ("efective", &[0]),
            ("enrgy", &[0]),
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

#[test]
fn test_latex_skips_markup_and_checks_prose() {
    // Citation keys, label references, environment names, class/package
    // options, command names, math, and macro definitions are markup, not
    // prose. A label's definition is the user's own name, so it's checked.
    let sample_text = r#"
\documentclass[journal,compsoc]{IEEEtran}
\usepackage[utf8]{inputenc}
\DeclareMathOperator{\FDD}{FDD}
\newcommand{\Mcal}{\mathcal{M}}
\begin{document}
\title{A Studdy of Things}
\author{Name,~\IEEEmembership{Member}}
\maketitle
\begin{IEEEkeywords} keyword, anothr \end{IEEEkeywords}
\section{Introductoin}
See \cite{smithh2020} and \ref{fig:resultz} for \textbf{detials}.
$\mathrm{abcz} \nonumber$
\begin{equation} \mathsf{qwerty} \label{eq:enrgy} \end{equation}
\includegraphics[width=\columnwidth]{figs/plott.pdf}
\end{document}
"#;
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[
            ("Studdy", &[0]),
            ("anothr", &[0]),
            ("Introductoin", &[0]),
            ("detials", &[0]),
            ("enrgy", &[0]),
        ],
    );
}

#[test]
fn test_latex_checks_text_arguments_of_reference_commands() {
    // Links, citations, and colour commands mix name-like arguments (URLs,
    // keys, colour names) with prose ones. Only the prose is checked.
    let sample_text = r"\href{http://example.com/pathh}{link texxt}
\url{http://example.com/othr}
\cite[see pagee][chap.~2]{smithh2020}
\textcolor{redd}{colord words}
\colorbox[rgb]{1,0,0}{boxd words}
\item[Labell] Item text.
";
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[
            ("Labell", &[0]),
            ("boxd", &[0]),
            ("colord", &[0]),
            ("pagee", &[0]),
            ("texxt", &[0]),
        ],
    );
}

#[test]
fn test_latex_checks_environment_titles_and_text_in_math() {
    // An environment's optional argument can be a title (prose) or a float
    // placement; `\text{}` and `\intertext{}` hold prose even inside math.
    let sample_text = r"\begin{theorem}[Mainn result]
Body text.
\end{theorem}
\begin{figure}[htbp]
\end{figure}
\begin{align}
a &= b \intertext{Betweenn the lines} c &= d
\end{align}
$x = 1 \text{if \textbf{boldd} and onlyy if} y$
";
    assert_spelling_at(
        LanguageType::Latex,
        sample_text,
        &[
            ("Betweenn", &[0]),
            ("Mainn", &[0]),
            ("boldd", &[0]),
            ("onlyy", &[0]),
        ],
    );
}
