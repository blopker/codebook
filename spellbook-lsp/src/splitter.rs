pub fn split_camel_case(input: &str) -> Vec<String> {
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
        assert_eq!(words, vec!["X", "M", "L", "Http", "Request"]);
    }
}
