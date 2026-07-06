use codebook::{parser::TextRange, queries::LanguageType};

/// Path relative to the crate directory, which is the cwd when cargo runs tests.
fn example_file_path(file: &str) -> String {
    format!("tests/examples/{file}")
}

#[test]
fn test_ignore_file() {
    let processor = super::utils::get_processor();
    let results = processor.spell_check("badword", None, Some("ignore.txt"));
    assert_eq!(results.len(), 0);
}

#[test]
fn test_include_paths_allowlist() {
    let processor = super::utils::get_processor_with_include("**/*.rs");
    assert!(
        !processor
            .spell_check("badword", Some(LanguageType::Text), Some("src/main.rs"))
            .is_empty()
    );
    assert!(
        processor
            .spell_check("badword", Some(LanguageType::Text), Some("src/main.py"))
            .is_empty()
    );
}

#[test]
fn test_include_paths_empty_includes_everything() {
    let processor = super::utils::get_processor();
    assert!(
        !processor
            .spell_check("badword", Some(LanguageType::Text), Some("src/main.rs"))
            .is_empty()
    );
    assert!(
        !processor
            .spell_check("badword", Some(LanguageType::Text), Some("src/main.py"))
            .is_empty()
    );
}

#[test]
fn test_ignore_paths_takes_precedence_over_include_paths() {
    let processor = super::utils::get_processor_with_include_and_ignore("**/*.rs", "**/*.rs");
    assert_eq!(
        processor
            .spell_check("badword", Some(LanguageType::Text), Some("src/main.rs"))
            .len(),
        0
    );
}

#[test]
fn test_example_files_word_locations() {
    // Each listed word occurs exactly once in its file (case-sensitive), so
    // the expected range is derived from the text instead of hand-written
    // byte offsets. `spell_check` also enforces the slice invariant.
    let files: &[(&str, &[&str])] = &[
        ("example.py", &["Pthon"]),
        ("example.ts", &["mistkes"]),
        ("example.txt", &["Splellin"]),
        ("example.md", &["wolrd", "Wolrd", "regulr"]),
    ];
    let processor = super::utils::get_processor();
    for (file, words) in files {
        let text = std::fs::read_to_string(example_file_path(file)).unwrap();
        let results = super::utils::spell_check(&processor, LanguageType::Text, &text);
        for word in *words {
            let starts: Vec<usize> = text.match_indices(word).map(|(i, _)| i).collect();
            assert_eq!(
                starts.len(),
                1,
                "'{word}' must occur exactly once in {file}"
            );
            let expected = vec![TextRange {
                start_byte: starts[0],
                end_byte: starts[0] + word.len(),
            }];
            let found = results
                .iter()
                .find(|r| r.word == *word)
                .unwrap_or_else(|| panic!("'{word}' was not flagged in {file}"));
            assert_eq!(found.locations, expected, "'{word}' in {file}");
        }
    }
}

#[test]
fn test_example_files() {
    // Exact (sorted) set of flagged words per file, checked through
    // `spell_check_file` so language detection from the path is exercised.
    let files: &[(&str, &[&str])] = &[
        ("example.html", &["Documentt", "Spelin", "Wolrd", "sor"]),
        ("example.py", &["Pthon", "Wolrd", "linest", "spelin"]),
        // The DNA sequence is flagged by default; the README documents
        // skipping such sequences via user-defined ignore_patterns.
        (
            "example.md",
            &["ATGCATCG", "Wolrd", "bvd", "regulr", "splellin", "wolrd"],
        ),
        ("example.txt", &["Splellin"]),
        ("example.rs", &["birt", "calclate", "curent", "jalopin"]),
        (
            "example.go",
            &["Alicz", "Funcion", "Wolrd", "alicz", "mispeled", "speling"],
        ),
        (
            "example.js",
            &[
                "Accaunt",
                "Calculater",
                "Exportt",
                "Funcshun",
                "Funktion",
                "Inputt",
                "Numbr",
                "Numbrs",
                "Pleese",
                "additshun",
                "arra",
                "calculater",
                "divde",
                "divishun",
                "emale",
                "funcsions",
                "inputt",
                "multiplacation",
                "numbr",
                "numbrs",
                "operashun",
                "passwrd",
                "propertys",
                "prosess",
                "resalt",
                "secand",
                "substractshun",
                "summ",
                "totel",
                "usege",
                "usrname",
            ],
        ),
        (
            "example.ts",
            &[
                "Accaunt",
                "Exportt",
                "Funcshun",
                "Funktion",
                "Inputt",
                "Numbr",
                "Numbrs",
                "Pleese",
                "arra",
                "emale",
                "funcsions",
                "inputt",
                "linet",
                "mistkes",
                "numbr",
                "numbrs",
                "passwrd",
                "propertys",
                "prosess",
                "secand",
                "totel",
                "usege",
                "usrname",
            ],
        ),
        (
            "example.lua",
            &[
                "Accont",
                "Helo",
                "Wrold",
                "calculat",
                "calculatr",
                "countr",
                "exampl",
                "intrest",
                "mesage",
                "numbr",
                "operashun",
            ],
        ),
    ];
    let processor = super::utils::get_processor();
    for (file, expected) in files {
        let results = processor
            .spell_check_file(&example_file_path(file))
            .unwrap();
        let mut misspelled: Vec<&str> = results.iter().map(|r| r.word.as_str()).collect();
        misspelled.sort_unstable();
        assert_eq!(&misspelled, expected, "flagged words in {file}");
    }
}
