use unicode_segmentation::UnicodeSegmentation;

#[derive(PartialEq)]
enum CharType {
    Lower,
    Upper,
    Digit,
    Underscore,
    Period,
    Colon,
}

#[derive(Debug, PartialEq)]
pub struct Split {
    pub word: String,
    pub start_char: u32,
}

pub fn split(s: &str) -> Vec<Split> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut current_word_start: usize = 0;
    let mut prev_char_type = None;

    for (i, c) in s.graphemes(true).enumerate() {
        assert!(
            !c.chars().any(|ch| ch.is_whitespace()),
            "There should be no white space in the input: '{}'",
            s
        );
        let char_type = c
            .chars()
            .map(|ch| match ch {
                ch if ch.is_uppercase() => CharType::Upper,
                ch if ch.is_ascii_digit() => CharType::Digit,
                '_' => CharType::Underscore,
                '.' => CharType::Period,
                ':' => CharType::Colon,
                _ => CharType::Lower,
            })
            .next()
            .unwrap();

        // Start a new word if:
        // 1. Current char is uppercase and previous was lowercase
        // 2. Current char is uppercase and next char is lowercase (for cases like "XML")
        // 3. Current char is a digit and previous was not
        // 4. Previous char was a digit and current is not
        // 5. Current char is an underscore
        // 5. Current char is a period
        let should_split = match prev_char_type {
            Some(CharType::Lower) if char_type == CharType::Upper => true,
            Some(CharType::Upper)
                if char_type == CharType::Upper && {
                    match s.chars().nth(i + 1) {
                        Some(t) => t.is_ascii_lowercase(),
                        None => false,
                    }
                } =>
            {
                true
            }
            Some(prev)
                if (prev != CharType::Digit && char_type == CharType::Digit)
                    || (prev == CharType::Digit && char_type != CharType::Digit) =>
            {
                true
            }
            _ => {
                char_type == CharType::Underscore
                    || char_type == CharType::Period
                    || char_type == CharType::Colon
            }
        };

        if should_split && !current_word.is_empty() {
            result.push(Split {
                word: current_word.clone(),
                start_char: current_word_start as u32,
            });
            current_word.clear();
            current_word_start = i;
        }
        if char_type == CharType::Underscore
            || char_type == CharType::Period
            || char_type == CharType::Colon
        {
            current_word_start = i + 1;
        } else {
            current_word += c;
        }
        prev_char_type = Some(char_type);
    }

    if !current_word.is_empty() {
        let start = s.chars().count() - current_word.chars().count();
        result.push(Split {
            word: current_word,
            start_char: start as u32,
        });
    }

    result
}

// pub fn find_url_end(text: &str) -> usize {
//     debug_assert!(text.starts_with("://"), "Input must start with '://'");

//     // Track nesting of parentheses, brackets, etc.
//     let mut paren_level = 0;
//     let mut bracket_level = 0;
//     let mut brace_level = 0;

//     // Track if we're inside a query string or fragment
//     let mut in_query_or_fragment = false;

//     // Examine each character to determine where URL ends
//     for (i, c) in text.char_indices() {
//         match c {
//             // Opening delimiters
//             '(' => paren_level += 1,
//             '[' => bracket_level += 1,
//             '{' => brace_level += 1,

//             // Closing delimiters
//             ')' => {
//                 if paren_level > 0 {
//                     paren_level -= 1;
//                 } else {
//                     // Unpaired closing parenthesis ends the URL
//                     return i;
//                 }
//             }
//             ']' => {
//                 if bracket_level > 0 {
//                     bracket_level -= 1;
//                 } else {
//                     return i;
//                 }
//             }
//             '}' => {
//                 if brace_level > 0 {
//                     brace_level -= 1;
//                 } else {
//                     return i;
//                 }
//             }

//             // Special URL components
//             '?' | '#' => in_query_or_fragment = true,

//             // Characters that typically end a URL
//             ' ' | '\t' | '\n' | '\r' | '"' | '\'' | '<' | '>' | '`' | '|' | '^' => return i,

//             // Punctuation that may end a URL unless in query/fragment
//             '.' | ',' | ':' | ';' | '!' => {
//                 if !in_query_or_fragment {
//                     // Look ahead to see if this is actually the end
//                     if let Some(next_char) = text[i + 1..].chars().next() {
//                         if next_char.is_whitespace() || "\"'<>()[]{}".contains(next_char) {
//                             return i;
//                         }
//                     } else {
//                         // End of string
//                         return i + 1;
//                     }
//                 }
//             }

//             // Other characters are allowed in the URL
//             _ => {}
//         }
//     }

//     // If we reach the end of the string, the URL extends to the end
//     text.len()
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_splitting() {
        let words: Vec<String> = split("calculateUserAge")
            .into_iter()
            .map(|s| s.word)
            .collect();
        assert_eq!(words, vec!["calculate", "User", "Age"]);
    }

    #[test]
    fn test_camel_case_splitting_underscore() {
        let words = split("calculateUser_Age____word__");
        assert_eq!(
            words,
            vec![
                Split {
                    word: "calculate".to_string(),
                    start_char: 0
                },
                Split {
                    word: "User".to_string(),
                    start_char: 9
                },
                Split {
                    word: "Age".to_string(),
                    start_char: 14
                },
                Split {
                    word: "word".to_string(),
                    start_char: 21
                }
            ]
        );
    }

    #[test]
    fn test_camel_case_splitting_period() {
        let words = split("calculateUser.Age.._.word._");
        assert_eq!(
            words,
            vec![
                Split {
                    word: "calculate".to_string(),
                    start_char: 0
                },
                Split {
                    word: "User".to_string(),
                    start_char: 9
                },
                Split {
                    word: "Age".to_string(),
                    start_char: 14
                },
                Split {
                    word: "word".to_string(),
                    start_char: 21
                }
            ]
        );
    }

    #[test]
    fn test_camel_case_splitting_colon() {
        let words = split("calculateUser:Age..:.word.:");
        assert_eq!(
            words,
            vec![
                Split {
                    word: "calculate".to_string(),
                    start_char: 0
                },
                Split {
                    word: "User".to_string(),
                    start_char: 9
                },
                Split {
                    word: "Age".to_string(),
                    start_char: 14
                },
                Split {
                    word: "word".to_string(),
                    start_char: 21
                }
            ]
        );
    }

    #[test]
    fn test_complex_camel_case() {
        let words = split("XMLHttpRequest");
        assert_eq!(
            words,
            vec![
                Split {
                    word: "XML".to_string(),
                    start_char: 0
                },
                Split {
                    word: "Http".to_string(),
                    start_char: 3
                },
                Split {
                    word: "Request".to_string(),
                    start_char: 7
                }
            ]
        );
    }

    #[test]
    fn test_number() {
        let words: Vec<String> = split("userAge10").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["user", "Age", "10"]);
    }

    #[test]
    fn test_uppercase() {
        let words: Vec<String> = split("EXAMPLE").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["EXAMPLE"]);
    }

    #[test]
    fn test_uppercase_first() {
        let words: Vec<String> = split("Example").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["Example"]);
    }

    #[test]
    fn test_unicode() {
        let words: Vec<String> = split("こんにちは").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["こんにちは"]);
    }

    // #[test]
    // fn test_find_url() {
    //     crate::log::init_test_logging();
    //     let text = "://example.com/path/to/file.html)not a url";
    //     let end = find_url_end(text);
    //     debug!("URL: {}", &text[..end]);
    //     assert_eq!(&text[..end], "://example.com/path/to/file.html");
    // }
}
