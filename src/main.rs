mod queries;
use lazy_static::lazy_static;
use queries::{get_language_name_from_filename, get_language_setting, LanguageSetting};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

lazy_static! {
    static ref EXTRA_WORDS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.extend(["http", "https", "www", "viewport"]);
        set
    };
}

#[derive(Debug, Clone)]
pub struct SpellCheckResult {
    pub word: String,
    pub suggestions: Vec<String>,
    pub locations: Vec<TextRange>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

pub struct WordProcessor {
    custom_dictionary: HashSet<String>,
    dictionary: spellbook::Dictionary,
}

impl WordProcessor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let aff = std::fs::read_to_string("index.aff")?;
        let dic = std::fs::read_to_string("index.dic")?;
        let dict = spellbook::Dictionary::new(&aff, &dic)
            .map_err(|e| format!("Dictionary parse error: {}", e))?;

        Ok(WordProcessor {
            custom_dictionary: HashSet::new(),
            dictionary: dict,
        })
    }

    pub fn add_to_dictionary(&mut self, word: String) {
        self.custom_dictionary.insert(word);
    }

    pub fn get_suggestions(&self, word: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        self.dictionary.suggest(word, &mut suggestions);
        suggestions
    }

    fn split_camel_case(&self, input: &str) -> Vec<String> {
        if input.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();
        let mut current_word = String::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                // Start of a new word with uppercase
                c if c.is_uppercase() => {
                    if !current_word.is_empty() {
                        result.push(current_word);
                        current_word = String::new();
                    }
                    current_word.push(chars.next().unwrap());
                }
                // Continue current word
                c if c.is_lowercase() || c.is_digit(10) => {
                    current_word.push(chars.next().unwrap());
                }
                // Skip other characters
                _ => {
                    chars.next();
                }
            }
        }

        if !current_word.is_empty() {
            result.push(current_word);
        }

        // Post-process to handle consecutive uppercase letters
        result
            .into_iter()
            .flat_map(|word| {
                if word.chars().all(|c| c.is_uppercase()) && word.len() > 1 {
                    word.chars().map(|c| c.to_string()).collect()
                } else {
                    vec![word]
                }
            })
            .collect()
    }

    fn should_skip_word(&self, word: &str) -> bool {
        if EXTRA_WORDS.contains(word) {
            return true;
        }

        if word.contains("://") || word.starts_with("www.") {
            return true; // Skip URLs
        }

        // Skip numbers, including those with type suffixes like u32
        if word.chars().next().map_or(false, |c| c.is_digit(10))
            || word.chars().any(|c| c.is_digit(10))
        {
            return true;
        }

        // Original checks
        if word.len() < 2
            || self.custom_dictionary.contains(word)
            || word.chars().all(|c| c.is_uppercase())
        {
            return true;
        }

        false
    }

    fn prepare_text_for_spell_check(&self, text: &str) -> HashSet<String> {
        let mut words_to_check = HashSet::new();

        // Split text into words and handle punctuation
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            if word.is_empty() || self.should_skip_word(word) {
                continue;
            }

            // Handle camelCase and PascalCase
            let parts = self.split_camel_case(word);

            for part in parts {
                if !self.should_skip_word(&part) {
                    words_to_check.insert(part);
                }
            }
        }

        words_to_check
    }

    pub fn spell_check(&self, text: &str, language: &str) -> Vec<SpellCheckResult> {
        println!("language: {:?}", language);
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

    fn spell_check_text(&self, text: &str) -> Vec<SpellCheckResult> {
        let words = self.prepare_text_for_spell_check(text);
        return words
            .into_iter()
            .filter(|word| !self.dictionary.check(word))
            .map(|word| SpellCheckResult {
                word: word.clone(),
                suggestions: self.get_suggestions(&word),
                locations: self.find_word_locations(&word, text),
            })
            .collect();
    }

    fn spell_check_code(
        &self,
        text: &str,
        language_setting: &LanguageSetting,
    ) -> Vec<SpellCheckResult> {
        // Set up parser for the specified language
        println!("Code check for {:?}", language_setting);
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

                let words_to_process = match capture.index as u32 {
                    0 => {
                        // identifier
                        if !node.is_named() || node.kind().contains("keyword") {
                            vec![]
                        } else {
                            self.node_text_to_parts(node_text)
                        }
                    }
                    1 | 2 | 3 => self.node_text_to_parts(node_text),
                    _ => continue,
                };
                println!("words_to_process: {words_to_process:?}");
                for word in words_to_process {
                    if !self.should_skip_word(&word) {
                        if !self.dictionary.check(&word) {
                            word_locations
                                .entry(word)
                                .or_insert_with(Vec::new)
                                .push(TextRange {
                                    start: node.start_byte(),
                                    end: node.end_byte(),
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
                suggestions: self.get_suggestions(&word),
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
                parts.extend(self.split_camel_case(word));
            }
        }
        parts
    }

    fn find_word_locations(&self, word: &str, text: &str) -> Vec<TextRange> {
        let mut locations = Vec::new();
        let matches = text.match_indices(word).collect::<Vec<_>>();
        for _match in matches {
            let start = _match.0;
            let end = start + word.len();
            locations.push(TextRange { start, end });
        }
        locations
    }
}

fn main() {
    let processor = WordProcessor::new().unwrap();
    let args: Vec<String> = env::args().collect();

    println!("My path is {:?}", args);
    if args.len() < 2 {
        let sample_text = r#"
            fn calculate_user_age(birthDate: String) -> u32 {
                // This is an example_function that calculates age
                let userAge = get_current_date() - birthDate;
                userAge
            }
        "#;

        let misspelled = processor.spell_check(sample_text, "rust");
        println!("Misspelled words: {:?}", misspelled);
        return;
    }
    let path = Path::new(args[1].as_str());
    if !path.exists() {
        eprintln!("Can't find file {path:?}");
        return;
    }
    let results = processor.spell_check_file(path.to_str().unwrap());
    println!("Misspelled words: {:?}", results);
    println!("Done");
}
//
// fn main() {
//     let stdin = io::stdin();
//     let mut stdout = io::stdout();

//     for line in stdin.lock().lines() {
//         if let Ok(json_rpc) = line {
//             // Parse JSON-RPC message here
//             match parse_json_rpc(&json_rpc) {
//                 Some(message) => handle_lsp_method(message, &mut stdout),
//                 None => eprintln!("Invalid JSON-RPC message"),
//             }
//         }
//     }
// }

// fn parse_json_rpc(json: &str) -> Option<String> {
//     // Implement your own parser or use a third-party library (like serde_json).
//     // For simplicity, this example does not include parsing.
//     eprintln!("{:?}", json);
//     None
// }

// fn handle_lsp_method(message: String, stdout: &mut dyn Write) {
//     // Handle LSP methods and send responses to the client here.
//     // This example does not include any LSP functionality.
//     writeln!(stdout, "{message}").unwrap(); // Send an empty response.
// }

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_camel_case_splitting() {
        let processor = WordProcessor::new();
        let words = processor.unwrap().split_camel_case("calculateUserAge");
        assert_eq!(words, vec!["calculate", "User", "Age"]);
    }

    #[test]
    fn test_spell_checking() {
        let processor = WordProcessor::new();

        let text = "HelloWorld calc_wrld";
        let misspelled = processor.unwrap().spell_check(text, "text");
        println!("{:?}", misspelled);
        assert!(misspelled.iter().any(|r| r.word == "wrld"));
    }

    #[test]
    fn test_complex_camel_case() {
        let processor = WordProcessor::new();
        let words = processor.unwrap().split_camel_case("XMLHttpRequest");
        assert_eq!(words, vec!["X", "M", "L", "Http", "Request"]);
    }

    #[test]
    fn test_programming() {
        let processor = WordProcessor::new().unwrap();
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
                    locations: vec![TextRange { start: 10, end: 18 }],
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
                        locations: vec![TextRange { start: 26, end: 31 }],
                    },
                    SpellCheckResult {
                        word: "Wolrd".to_string(),
                        suggestions: vec![
                            "World".to_string(),
                            "Word".to_string(),
                            "Wold".to_string(),
                        ],
                        locations: vec![TextRange { start: 20, end: 25 }],
                    },
                ],
            ),
        ];
        for file in files {
            let path = format!("examples/{}", file.0);
            println!("Checking file: {path:?}");
            let text = std::fs::read_to_string(path).unwrap();
            let processor = WordProcessor::new().unwrap();
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
            ("example.md", vec!["Wolrd", "bvd", "splellin", "wolrd"]),
            ("example.txt", vec!["Splellin", "bd"]),
            ("example.rs", vec!["birt", "curent", "jalopin", "usr"]),
            ("example.go", vec!["speling", "Wolrd", "mispeled"]),
        ];
        for file in files {
            let path = format!("examples/{}", file.0);
            println!("Checking file: {path:?}");
            let processor = WordProcessor::new().unwrap();
            let results = processor.spell_check_file(&path);
            let mut misspelled = results
                .iter()
                .map(|r| r.word.as_str())
                .collect::<Vec<&str>>();
            misspelled.sort();
            println!("Misspelled words: {misspelled:?}");
            assert_eq!(misspelled, file.1);
        }
    }
}
