use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

fn example_file_path(file: &str) -> String {
    // get root of the project through CARGO_MANIFEST_DIR
    format!("tests/examples/{file}")
}

#[test]
fn test_ignore_file() {
    utils::init_logging();
    let processor = utils::get_processor();
    let results = processor.spell_check("badword", None, Some("ignore.txt"));
    assert_eq!(results.len(), 0);
}

#[test]
fn test_example_files_word_locations() {
    utils::init_logging();
    let files: Vec<(&str, Vec<WordLocation>)> = vec![
        (
            "example.py",
            vec![WordLocation::new(
                "Pthon".to_string(),
                vec![TextRange {
                    start_char: 10,
                    end_char: 15,
                    line: 0,
                }],
            )],
        ),
        (
            "example.ts",
            vec![WordLocation::new(
                "mistkes".to_string(),
                vec![TextRange {
                    start_char: 19,
                    end_char: 26,
                    line: 12,
                }],
            )],
        ),
        // ("example.md", vec!["bvd", "splellin", "wolrd"]),
        (
            "example.txt",
            vec![WordLocation {
                word: "Splellin".to_string(),
                locations: vec![TextRange {
                    start_char: 10,
                    end_char: 18,
                    line: 0,
                }],
            }],
        ),
        (
            "example.md",
            vec![
                WordLocation {
                    word: "wolrd".to_string(),
                    locations: vec![TextRange {
                        start_char: 26,
                        end_char: 31,
                        line: 0,
                    }],
                },
                WordLocation {
                    word: "Wolrd".to_string(),
                    locations: vec![TextRange {
                        start_char: 20,
                        end_char: 25,
                        line: 0,
                    }],
                },
                WordLocation {
                    word: "regulr".to_string(),
                    locations: vec![TextRange {
                        start_char: 6,
                        end_char: 12,
                        line: 1,
                    }],
                },
            ],
        ),
    ];
    for file in files {
        let path = example_file_path(file.0);
        println!("Checking file: {path:?}");
        let text = std::fs::read_to_string(path).unwrap();
        let processor = utils::get_processor();
        let results = processor.spell_check(&text, Some(LanguageType::Text), None);
        println!("Misspelled words: {results:?}");
        for expected in file.1 {
            let found = results.iter().find(|r| r.word == expected.word).unwrap();
            assert_eq!(found.locations, expected.locations);
        }
    }
}

#[test]
fn test_example_files() {
    utils::init_logging();
    let files = [
        ("example.html", vec!["Spelin", "Wolrd", "sor"]),
        ("example.py", vec!["Pthon", "Wolrd"]),
        (
            "example.md",
            vec!["Wolrd", "bvd", "regulr", "splellin", "wolrd"],
        ),
        ("example.txt", vec!["Splellin"]),
        ("example.rs", vec!["birt", "calclate", "curent", "jalopin"]),
        (
            "example.go",
            vec!["speling", "Wolrd", "mispeled", "Funcion"],
        ),
        (
            "example.js",
            vec![
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
            ],
        ),
        (
            "example.ts",
            vec![
                "Accaunt", "Exportt", "Funcshun", "Funktion", "Inputt", "Numbr", "Numbrs",
            ],
        ),
        (
            "example.lua",
            vec![
                "exampl",
                "Helo",
                "Wrold",
                "mesage",
                "countr",
                "Accont",
                "calculat",
                "intrest",
                "calculatr",
                "numbr",
                "operashun",
            ],
        ),
    ];
    for mut file in files {
        let path = example_file_path(file.0);
        println!("---------- Checking file: {path:?} ----------");
        let processor = utils::get_processor();
        let results = processor.spell_check_file(&path);
        let mut misspelled = results
            .iter()
            .map(|r| r.word.as_str())
            .collect::<Vec<&str>>();
        misspelled.sort();
        file.1.sort();
        println!("Misspelled words: {misspelled:?}");
        for word in &file.1 {
            assert!(misspelled.contains(word), "Word: {}", word);
        }
    }
}
