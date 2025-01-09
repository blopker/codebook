pub mod downloader;
mod queries;
mod splitter;

use crate::queries::{get_language_name_from_filename, get_language_setting, LanguageSetting};
use std::collections::{HashMap, HashSet};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug, Clone)]
pub struct SpellCheckResult {
    pub word: String,
    pub suggestions: Vec<String>,
    pub locations: Vec<TextRange>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextRange {
    pub start_char: u32,
    pub end_char: u32,
    pub start_line: u32,
    pub end_line: u32,
}

pub struct CodeDictionary {
    custom_dictionary: HashSet<String>,
    dictionary: spellbook::Dictionary,
}

impl CodeDictionary {
    pub fn new(aff_path: &str, dic_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let aff = std::fs::read_to_string(aff_path)?;
        let dic = std::fs::read_to_string(dic_path)?;
        let dict = spellbook::Dictionary::new(&aff, &dic)
            .map_err(|e| format!("Dictionary parse error: {}", e))?;

        Ok(CodeDictionary {
            custom_dictionary: HashSet::new(),
            dictionary: dict,
        })
    }

    pub fn add_to_dictionary(&mut self, word: String) {
        self.custom_dictionary.insert(word);
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        self.dictionary.suggest(word, &mut suggestions);
        suggestions
    }

    pub fn spell_check(&self, text: &str, language: &str) -> Vec<SpellCheckResult> {
        // println!("language: {:?}", language);
        let lang = get_language_setting(language);
        match lang {
            None => {
                return self.spell_check_text(text);
            }
            Some(lang) => {
                return self.spell_check_code(text, lang);
            }
        }
    }

    pub fn spell_check_file(&self, path: &str) -> Vec<SpellCheckResult> {
        let lang_name = get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path).unwrap();
        return self.spell_check(&file_text, &lang_name);
    }

    pub fn spell_check_file_memory(&self, path: &str, contents: &str) -> Vec<SpellCheckResult> {
        let lang_name = get_language_name_from_filename(path);
        return self.spell_check(&contents, &lang_name);
    }

    fn spell_check_text(&self, text: &str) -> Vec<SpellCheckResult> {
        let words = splitter::split_into_words(text);
        let mut results: Vec<SpellCheckResult> = words
            .into_iter()
            .filter(|word| !self.dictionary.check(word))
            .map(|word| SpellCheckResult {
                word: word.clone(),
                suggestions: self.suggest(&word),
                locations: self.find_word_locations(&word, text),
            })
            .collect();
        results.sort_by(|a, b| a.word.cmp(&b.word));
        results
    }

    fn spell_check_code(
        &self,
        text: &str,
        language_setting: &LanguageSetting,
    ) -> Vec<SpellCheckResult> {
        // Set up parser for the specified language
        // println!("Code check for {:?}", language_setting);
        let mut parser = Parser::new();
        let language = language_setting.language().unwrap();
        parser.set_language(&language).unwrap();

        let tree = parser.parse(text, None).unwrap();
        let root_node = tree.root_node();

        let query = Query::new(&language, language_setting.query).unwrap();
        let mut cursor = QueryCursor::new();
        let mut word_locations = HashMap::new();
        let mut matches_query = cursor.matches(&query, root_node, text.as_bytes());

        // Process matches
        while let Some(match_) = matches_query.next() {
            for capture in match_.captures {
                let node = capture.node;
                let node_text = node.utf8_text(text.as_bytes()).unwrap();
                self.node_text_to_parts(node_text);
                let words_to_process = self.node_text_to_parts(node_text);
                println!("words_to_process: {words_to_process:?}");
                for word in words_to_process {
                    if !self.custom_dictionary.contains(&word) {
                        if !self.dictionary.check(&word) {
                            word_locations
                                .entry(word)
                                .or_insert_with(Vec::new)
                                .push(TextRange {
                                    start_char: u32::try_from(node.start_position().column)
                                        .unwrap(),
                                    end_char: u32::try_from(node.end_position().column).unwrap(),
                                    start_line: u32::try_from(node.start_position().row).unwrap(),
                                    end_line: u32::try_from(node.end_position().row).unwrap(),
                                });
                        }
                    }
                }
            }
        }

        // Check spelling and collect results
        word_locations
            .keys()
            .into_iter()
            .map(|word| SpellCheckResult {
                word: word.clone(),
                suggestions: self.suggest(&word),
                locations: word_locations.get(word).cloned().unwrap_or_default(),
            })
            .collect()
    }

    fn node_text_to_parts(&self, node_text: &str) -> Vec<String> {
        // string literal or comments
        // Split identifiers into parts
        let mut parts = Vec::new();
        // First split by non-alphanumeric
        println!("node_text: {node_text:?}");
        for word in node_text.split(|c: char| !c.is_alphanumeric()) {
            if !word.is_empty() {
                // Then split camelCase
                parts.extend(splitter::split_camel_case(word));
            }
        }
        parts
    }

    fn find_word_locations(&self, word: &str, text: &str) -> Vec<TextRange> {
        let mut locations = Vec::new();
        let mut line = 0;
        for line_text in text.lines() {
            let mut start = 0;
            while let Some(start_index) = line_text[start..].find(word) {
                start = start + start_index;
                let end = start + word.len();
                locations.push(TextRange {
                    start_char: u32::try_from(start).unwrap(),
                    end_char: u32::try_from(end).unwrap(),
                    start_line: line,
                    end_line: line,
                });
                start = end;
            }
            line += 1;
        }
        locations
    }
}

#[cfg(test)]
mod lib_tests {
    use super::*;
    static EXTRA_WORDS: &'static [&'static str] = &["http", "https", "www", "viewport", "UTF"];
    fn example_file_path(file: &str) -> String {
        format!("../examples/{}", file)
    }
    fn get_processor() -> CodeDictionary {
        let mut cdict =
            CodeDictionary::new("./tests/en_index.aff", "./tests/en_index.dic").unwrap();
        for word in EXTRA_WORDS {
            cdict.add_to_dictionary(word.to_string());
        }
        cdict
    }
    // #[test]
    // fn test_spell() {
    //     let aff = std::fs::read_to_string("index.aff").unwrap();
    //     let dic = std::fs::read_to_string("index.dic").unwrap();
    //     let dict = spellbook::Dictionary::new(&aff, &dic).unwrap();
    //     let mut suggestions: Vec<String> = Vec::new();
    //     dict.suggest("helloWorld", &mut suggestions);
    //     println!("{:?}", suggestions);
    //     assert!(false);
    // }

    #[test]
    fn test_spell_checking() {
        let processor = get_processor();

        let text = "HelloWorld calc_wrld";
        let misspelled = processor.spell_check(text, "text");
        println!("{:?}", misspelled);
        assert!(misspelled.iter().any(|r| r.word == "wrld"));
    }

    #[test]
    fn test_programming() {
        let processor = get_processor();
        let sample_text = r#"
            fn calculate_user_age(birthDate: String) -> u32 {
                // This is an example_function that calculates age
                let userAge = get_curent_date() - bithDate;
                userAge
            }
        "#;
        let expected = vec!["bith", "curent"];
        let binding = processor.spell_check(sample_text, "rust").to_vec();
        let mut misspelled = binding
            .iter()
            .map(|r| r.word.as_str())
            .collect::<Vec<&str>>();
        misspelled.sort();
        println!("Misspelled words: {misspelled:?}");
        assert_eq!(misspelled, expected);
    }

    #[test]
    fn test_example_files_word_locations() {
        let files: Vec<(&str, Vec<SpellCheckResult>)> = vec![
            // ("example.py", vec!["pthon", "wolrd"]),
            // ("example.html", vec!["sor", "spelin", "wolrd"]),
            // ("example.md", vec!["bvd", "splellin", "wolrd"]),
            (
                "example.txt",
                vec![SpellCheckResult {
                    word: "Splellin".to_string(),
                    suggestions: vec![
                        "Spelling".to_string(),
                        "Spline".to_string(),
                        "Spineless".to_string(),
                    ],
                    locations: vec![TextRange {
                        start_char: 10,
                        end_char: 18,
                        start_line: 0,
                        end_line: 0,
                    }],
                }],
            ),
            (
                "example.md",
                vec![
                    SpellCheckResult {
                        word: "wolrd".to_string(),
                        suggestions: vec![
                            "world".to_string(),
                            "word".to_string(),
                            "wold".to_string(),
                        ],
                        locations: vec![TextRange {
                            start_char: 26,
                            end_char: 31,
                            start_line: 0,
                            end_line: 0,
                        }],
                    },
                    SpellCheckResult {
                        word: "Wolrd".to_string(),
                        suggestions: vec![
                            "World".to_string(),
                            "Word".to_string(),
                            "Wold".to_string(),
                        ],
                        locations: vec![TextRange {
                            start_char: 20,
                            end_char: 25,
                            start_line: 0,
                            end_line: 0,
                        }],
                    },
                    SpellCheckResult {
                        word: "regulr".to_string(),
                        suggestions: vec!["regular".to_string(), "Regulus".to_string()],
                        locations: vec![TextRange {
                            start_char: 6,
                            end_char: 12,
                            start_line: 1,
                            end_line: 1,
                        }],
                    },
                ],
            ),
        ];
        for file in files {
            let path = example_file_path(file.0);
            println!("Checking file: {path:?}");
            let text = std::fs::read_to_string(path).unwrap();
            let processor = get_processor();
            let results = processor.spell_check(&text, "text");
            println!("Misspelled words: {results:?}");
            for expected in file.1 {
                let found = results.iter().find(|r| r.word == expected.word).unwrap();
                assert_eq!(found.suggestions, expected.suggestions);
                assert_eq!(found.locations, expected.locations);
            }
        }
    }

    #[test]
    fn test_example_files() {
        let files = [
            ("example.html", vec!["Spelin", "Wolrd", "sor"]),
            ("example.py", vec!["Pthon", "Wolrd"]),
            (
                "example.md",
                vec!["Wolrd", "bvd", "regulr", "splellin", "wolrd"],
            ),
            ("example.txt", vec!["Splellin", "bd"]),
            ("example.rs", vec!["birt", "curent", "jalopin", "usr"]),
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
                    "ballance",
                    "calculater",
                    "divde",
                    "divishun",
                    "emale",
                    "funcsions",
                    "inputt",
                    "lenght",
                    "logg",
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
                vec![
                    "Accaunt",
                    "Exportt",
                    "Funcshun",
                    "Funktion",
                    "Inputt",
                    "Numbr",
                    "Numbrs",
                    "Pleese",
                    "arra",
                    "ballance",
                    "emale",
                    "funcsions",
                    "inputt",
                    "logg",
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
        ];
        for mut file in files {
            let path = example_file_path(file.0);
            println!("Checking file: {path:?}");
            let processor = get_processor();
            let results = processor.spell_check_file(&path);
            let mut misspelled = results
                .iter()
                .map(|r| r.word.as_str())
                .collect::<Vec<&str>>();
            misspelled.sort();
            file.1.sort();
            println!("Misspelled words: {misspelled:?}");
            assert_eq!(misspelled, file.1);
        }
    }
}
