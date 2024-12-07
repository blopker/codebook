use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Language, Parser, Query, QueryCursor};

lazy_static! {
    static ref WORD_BOUNDARY_RE: Regex = Regex::new(r"[A-Z]?[a-z]+|[A-Z]+(?:[A-Z][a-z]+)*|\d+").unwrap();

    // Expanded programming terms
    static ref PROGRAMMING_TERMS: HashSet<&'static str> = {
        let mut terms = HashSet::new();
        // Basic types and concepts
        terms.extend(&["str", "int", "bool", "dict", "list", "func", "var", "const",
                      "enum", "impl", "struct", "len", "repr", "vec", "ref", "mut",
                      "async", "await", "static", "self", "super", "trait"]);
        // Common abbreviations
        terms.extend(&["arg", "args", "param", "params", "func", "ptr", "impl", "iter",
                      "prev", "curr", "num", "calc", "tmp", "temp", "init", "dest", "src"]);
        // Web/Network terms
        terms.extend(&["http", "https", "www", "html", "css", "js", "url", "uri", "api",
                      "rest", "xml", "json", "sql", "dto", "tcp", "udp", "ip"]);
        // File extensions
        terms.extend(&["rs", "txt", "md", "yml", "toml", "json", "js", "ts", "py"]);
        terms
    };

    static ref RUST_KEYWORDS: HashSet<&'static str> = {
        let mut keywords = HashSet::new();
        keywords.extend(&["fn", "let", "mut", "pub", "struct", "enum", "trait", "impl",
                         "type", "mod", "use", "where", "for", "match", "if", "else",
                         "loop", "while", "break", "continue", "return", "self", "Self",
                         "super", "move", "box", "in", "extern", "crate", "unsafe"]);
        keywords
    };

    static ref HTML_KEYWORDS: HashSet<&'static str> = {
        let mut keywords = HashSet::new();
        keywords.extend(&["charset", "lang", "viewport"]);
        keywords
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

    fn language_from_name(&self, name: &str) -> Option<Language> {
        match name {
            "rust" => Some(tree_sitter_rust::language()),
            "python" => Some(tree_sitter_python::language()),
            "javascript" => Some(tree_sitter_javascript::language()),
            _ => None,
        }
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
        let word_lower = word.to_lowercase();

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
            || PROGRAMMING_TERMS.contains(word_lower.as_str())
            || RUST_KEYWORDS.contains(word_lower.as_str())
            || HTML_KEYWORDS.contains(word_lower.as_str())
            || self.custom_dictionary.contains(&word_lower)
            || word.chars().all(|c| c.is_uppercase())
        {
            return true;
        }

        false
    }

    pub fn prepare_text_for_spell_check(&self, text: &str) -> HashSet<String> {
        let mut words_to_check = HashSet::new();

        // Split text into words and handle punctuation
        for word in text.split(|c: char| !c.is_alphanumeric()) {
            if word.is_empty() || self.should_skip_word(word) {
                continue;
            }

            // Handle camelCase and PascalCase
            let parts = self.split_camel_case(word);

            for part in parts {
                let lower_part = part.to_lowercase();
                if !self.should_skip_word(&lower_part) {
                    words_to_check.insert(lower_part);
                }
            }
        }

        words_to_check
    }

    pub fn spell_check(&self, text: &str, language: &str) -> Vec<SpellCheckResult> {
        let lang = self.language_from_name(language);
        match lang {
            None => {
                return self.spell_check_text(text);
            }
            Some(lang) => {
                return self.spell_check_code(text, lang);
            }
        }
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

    fn spell_check_code(&self, text: &str, language: Language) -> Vec<SpellCheckResult> {
        // Set up parser for the specified language
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        let tree = parser.parse(text, None).unwrap();
        let root_node = tree.root_node();

        let query = Query::new(
            tree_sitter_rust::language(),
            r#"
              (identifier) @identifier
              (string_literal) @string
              (line_comment) @comment
              (block_comment) @comment
              "#,
        )
        .unwrap();

        let mut cursor = QueryCursor::new();
        let mut words_to_check = HashSet::new();
        let mut word_locations = HashMap::new();

        // Process matches
        for match_ in cursor.matches(&query, root_node, text.as_bytes()) {
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
                    let lower_word = word.to_lowercase();
                    if !self.should_skip_word(&lower_word) {
                        words_to_check.insert(lower_word.clone());
                        word_locations
                            .entry(lower_word)
                            .or_insert_with(Vec::new)
                            .push(TextRange {
                                start: node.start_byte(),
                                end: node.end_byte(),
                            });
                    }
                }
            }
        }

        // Check spelling and collect results
        words_to_check
            .into_iter()
            .filter(|word| !self.dictionary.check(word))
            .map(|word| SpellCheckResult {
                word: word.clone(),
                suggestions: self.get_suggestions(&word),
                locations: word_locations.get(&word).cloned().unwrap_or_default(),
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
        let word_lower = word.to_lowercase();
        let mut pos = 0;

        for line in text.lines() {
            for word_match in line.split(|c: char| !c.is_alphanumeric()) {
                if !word_match.is_empty() {
                    // Check the word and its parts
                    let parts = self.split_camel_case(word_match);
                    for part in parts {
                        if part.to_lowercase() == word_lower {
                            let start = pos + line.find(word_match).unwrap_or(0);
                            let end = start + word_match.len();
                            locations.push(TextRange { start, end });
                        }
                    }
                }
            }
            pos += line.len() + 1; // +1 for newline
        }

        locations
    }
}

fn main() {
    let processor = WordProcessor::new();

    let sample_text = r#"
        fn calculate_user_age(birthDate: String) -> u32 {
            // This is an example_function that calculates age
            let userAge = get_current_date() - birthDate;
            userAge
        }
    "#;

    let misspelled = processor.unwrap().spell_check(sample_text, "rust");
    println!("Misspelled words: {:?}", misspelled);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let files = [
            // ("example.py", vec!["pthon", "wolrd"]),
            // ("example.html", vec!["sor", "spelin", "wolrd"]),
            // ("example.md", vec!["bvd", "splellin", "wolrd"]),
            (
                "example.txt",
                [SpellCheckResult {
                    word: "splellin".to_string(),
                    suggestions: vec![
                        "spelling".to_string(),
                        "spline".to_string(),
                        "spineless".to_string(),
                    ],
                    locations: vec![TextRange { start: 10, end: 18 }],
                }],
            ),
            (
                "example.md",
                [SpellCheckResult {
                    word: "wolrd".to_string(),
                    suggestions: vec!["world".to_string(), "word".to_string(), "wold".to_string()],
                    locations: vec![
                        TextRange { start: 20, end: 25 },
                        TextRange { start: 26, end: 31 },
                    ],
                }],
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
            ("example.py", vec!["pthon", "wolrd"]),
            ("example.html", vec!["sor", "spelin", "wolrd"]),
            ("example.md", vec!["bvd", "splellin", "wolrd"]),
            ("example.txt", vec!["bd", "splellin"]),
        ];
        for file in files {
            let path = format!("examples/{}", file.0);
            println!("Checking file: {path:?}");
            let text = std::fs::read_to_string(path).unwrap();
            let processor = WordProcessor::new().unwrap();
            let results = processor.spell_check(&text, "text");
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
