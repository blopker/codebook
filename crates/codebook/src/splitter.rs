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
pub struct SplitRef<'a> {
    pub word: &'a str,
    pub start_byte: usize,
}

pub fn split(s: &str) -> Vec<SplitRef<'_>> {
    if s.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut word_start_byte = 0;
    let mut prev_char_type = None;

    let mut char_iter = s.char_indices().peekable();

    while let Some((byte_pos, c)) = char_iter.next() {
        assert!(
            !c.is_whitespace(),
            "There should be no white space in the input: '{s}'"
        );

        let char_type = match c {
            ch if ch.is_uppercase() => CharType::Upper,
            ch if ch.is_ascii_digit() => CharType::Digit,
            '_' => CharType::Underscore,
            '.' => CharType::Period,
            ':' => CharType::Colon,
            _ => CharType::Lower,
        };

        let should_split = match prev_char_type {
            Some(CharType::Lower) if char_type == CharType::Upper => true,
            Some(CharType::Upper) if char_type == CharType::Upper => char_iter
                .peek()
                .map(|(_, next_c)| next_c.is_ascii_lowercase())
                .unwrap_or(false),
            Some(prev)
                if (prev != CharType::Digit && char_type == CharType::Digit)
                    || (prev == CharType::Digit && char_type != CharType::Digit) =>
            {
                true
            }
            _ => matches!(
                char_type,
                CharType::Underscore | CharType::Period | CharType::Colon
            ),
        };

        if should_split && byte_pos > word_start_byte {
            let word_slice = &s[word_start_byte..byte_pos];
            if !word_slice.is_empty() && !word_slice.chars().all(|c| matches!(c, '_' | '.' | ':')) {
                result.push(SplitRef {
                    word: word_slice,
                    start_byte: word_start_byte,
                });
            }
            word_start_byte = byte_pos;
        }

        if matches!(
            char_type,
            CharType::Underscore | CharType::Period | CharType::Colon
        ) {
            if let Some((next_byte_pos, _)) = char_iter.peek() {
                word_start_byte = *next_byte_pos;
            } else {
                word_start_byte = s.len();
            }
        }

        prev_char_type = Some(char_type);
    }

    // Handle final word
    if word_start_byte < s.len() {
        let word_slice = &s[word_start_byte..];
        if !word_slice.is_empty() && !word_slice.chars().all(|c| matches!(c, '_' | '.' | ':')) {
            result.push(SplitRef {
                word: word_slice,
                start_byte: word_start_byte,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_splitting() {
        let words: Vec<&str> = split("calculateUserAge")
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
                SplitRef {
                    word: "calculate",
                    start_byte: 0
                },
                SplitRef {
                    word: "User",
                    start_byte: 9
                },
                SplitRef {
                    word: "Age",
                    start_byte: 14
                },
                SplitRef {
                    word: "word",
                    start_byte: 21
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
                SplitRef {
                    word: "calculate",
                    start_byte: 0
                },
                SplitRef {
                    word: "User",
                    start_byte: 9
                },
                SplitRef {
                    word: "Age",
                    start_byte: 14
                },
                SplitRef {
                    word: "word",
                    start_byte: 21
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
                SplitRef {
                    word: "calculate",
                    start_byte: 0
                },
                SplitRef {
                    word: "User",
                    start_byte: 9
                },
                SplitRef {
                    word: "Age",
                    start_byte: 14
                },
                SplitRef {
                    word: "word",
                    start_byte: 21
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
                SplitRef {
                    word: "XML",
                    start_byte: 0
                },
                SplitRef {
                    word: "Http",
                    start_byte: 3
                },
                SplitRef {
                    word: "Request",
                    start_byte: 7
                }
            ]
        );
    }

    #[test]
    fn test_number() {
        let words: Vec<&str> = split("userAge10").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["user", "Age", "10"]);
    }

    #[test]
    fn test_uppercase() {
        let words: Vec<&str> = split("EXAMPLE").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["EXAMPLE"]);
    }

    #[test]
    fn test_uppercase_first() {
        let words: Vec<&str> = split("Example").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["Example"]);
    }

    #[test]
    fn test_unicode() {
        let words: Vec<&str> = split("こんにちは").into_iter().map(|s| s.word).collect();
        assert_eq!(words, vec!["こんにちは"]);
    }

    #[test]
    fn test_split_ref_camel_case() {
        let words = split("calculateUserAge");
        assert_eq!(
            words,
            vec![
                SplitRef {
                    word: "calculate",
                    start_byte: 0
                },
                SplitRef {
                    word: "User",
                    start_byte: 9
                },
                SplitRef {
                    word: "Age",
                    start_byte: 13
                }
            ]
        );
    }

    #[test]
    fn test_split_ref_with_separators() {
        let words = split("calculateUser_Age__word");
        assert_eq!(
            words,
            vec![
                SplitRef {
                    word: "calculate",
                    start_byte: 0
                },
                SplitRef {
                    word: "User",
                    start_byte: 9
                },
                SplitRef {
                    word: "Age",
                    start_byte: 14
                },
                SplitRef {
                    word: "word",
                    start_byte: 19
                }
            ]
        );
    }

    #[test]
    fn test_split_ref_xml_case() {
        let words = split("XMLHttpRequest");
        assert_eq!(
            words,
            vec![
                SplitRef {
                    word: "XML",
                    start_byte: 0
                },
                SplitRef {
                    word: "Http",
                    start_byte: 3
                },
                SplitRef {
                    word: "Request",
                    start_byte: 7
                }
            ]
        );
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
