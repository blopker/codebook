use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashSet, path::Path};

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
}

#[derive(Debug, Clone)]
pub struct SpellCheckResult {
    pub word: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug)]
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
        let word_lower = word.to_lowercase();

        // Additional checks
        if word.contains('.') && Path::new(word).exists() {
            return true; // Skip file paths
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
            || PROGRAMMING_TERMS.contains(word_lower.as_str())
            || RUST_KEYWORDS.contains(word_lower.as_str())
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

    pub fn spell_check_code(&self, text: &str) -> Vec<SpellCheckResult> {
        let words = self.prepare_text_for_spell_check(text);
        println!("words:{words:?}");
        let mut results = Vec::new();

        for word in words {
            if !self.dictionary.check(&word) {
                let suggestions = self.get_suggestions(&word);
                results.push(SpellCheckResult { word, suggestions });
            }
        }

        results
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

    let misspelled = processor.unwrap().spell_check_code(sample_text);
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
        let misspelled = processor.unwrap().spell_check_code(text);
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
        let binding = processor.spell_check_code(sample_text).to_vec();
        let misspelled = binding
            .iter()
            .map(|r| r.word.as_str())
            .collect::<Vec<&str>>();
        println!("Misspelled words: {misspelled:?}");
        assert_eq!(misspelled, expected);
    }
}
