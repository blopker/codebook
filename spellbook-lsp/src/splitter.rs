#[derive(PartialEq)]
enum CharType {
    Lower,
    Upper,
    Digit,
}

pub fn split_camel_case(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_word = String::new();
    let mut prev_char_type = None;

    for (i, c) in s.chars().enumerate() {
        let char_type = if c.is_ascii_uppercase() {
            CharType::Upper
        } else if c.is_ascii_digit() {
            CharType::Digit
        } else {
            CharType::Lower
        };

        // Start a new word if:
        // 1. Current char is uppercase and previous was lowercase
        // 2. Current char is uppercase and next char is lowercase (for cases like "XML")
        // 3. Current char is a digit and previous was not
        // 4. Previous char was a digit and current is not
        let should_split = match prev_char_type {
            Some(CharType::Lower) if char_type == CharType::Upper => true,
            Some(CharType::Upper)
                if char_type == CharType::Upper
                    && s.chars()
                        .nth(i + 1)
                        .map_or(false, |next| next.is_ascii_lowercase()) =>
            {
                true
            }
            Some(prev)
                if (prev != CharType::Digit && char_type == CharType::Digit)
                    || (prev == CharType::Digit && char_type != CharType::Digit) =>
            {
                true
            }
            _ => false,
        };

        if should_split && !current_word.is_empty() {
            result.push(current_word);
            current_word = String::new();
        }

        current_word.push(c);
        prev_char_type = Some(char_type);
    }

    if !current_word.is_empty() {
        result.push(current_word);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_splitting() {
        let words = split_camel_case("calculateUserAge");
        assert_eq!(words, vec!["calculate", "User", "Age"]);
    }

    #[test]
    fn test_complex_camel_case() {
        let words = split_camel_case("XMLHttpRequest");
        assert_eq!(words, vec!["XML", "Http", "Request"]);
    }

    #[test]
    fn test_number() {
        let words = split_camel_case("userAge10");
        assert_eq!(words, vec!["user", "Age", "10"]);
    }
}
