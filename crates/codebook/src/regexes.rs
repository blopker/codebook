use regex::Regex;

/// Default regex patterns to skip during spell checking.
/// These patterns match common technical strings that contain letter sequences
/// but shouldn't be treated as words for spell checking purposes.
pub fn get_default_skip_patterns() -> Vec<Regex> {
    vec![
        // URLs (http/https)
        Regex::new(r"https?://[^\s]+").expect("Valid URL regex"),
        // Hex colors (#deadbeef, #fff, #123456)
        Regex::new(r"#[0-9a-fA-F]{3,8}").expect("Valid hex color regex"),
        // Email addresses
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").expect("Valid email regex"),
        // File paths (Unix-style starting with /)
        Regex::new(r"/[^\s]*").expect("Valid Unix path regex"),
        // File paths (Windows-style with drive letter)
        Regex::new(r"[A-Za-z]:\\[^\s]*").expect("Valid Windows path regex"),
        // UUID
        Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
            .expect("Valid UUID regex"),
        // Base64 strings (rough pattern for long base64 sequences)
        Regex::new(r"[A-Za-z0-9+/]{20,}={0,2}").expect("Valid Base64 regex"),
        // Git commit hashes (7+ hex characters)
        Regex::new(r"\b[0-9a-fA-F]{7,40}\b").expect("Valid git hash regex"),
        // Markdown/HTML links
        Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("Valid markdown link regex"),
    ]
}

/// Compile user-provided regex patterns from strings
pub fn compile_user_patterns(patterns: &[String]) -> Result<Vec<Regex>, regex::Error> {
    patterns.iter().map(|pattern| Regex::new(pattern)).collect()
}

/// Combine default and user patterns into a single vector
pub fn get_combined_patterns(user_patterns: &[String]) -> Vec<Regex> {
    let mut patterns = get_default_skip_patterns();

    if let Ok(user_regexes) = compile_user_patterns(user_patterns) {
        patterns.extend(user_regexes);
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_pattern() {
        let patterns = get_default_skip_patterns();
        let url_pattern = &patterns[0]; // First pattern should be URLs

        assert!(url_pattern.is_match("https://www.example.com"));
        assert!(url_pattern.is_match("http://github.com/user/repo"));
        assert!(!url_pattern.is_match("not a url"));
    }

    #[test]
    fn test_hex_color_pattern() {
        let patterns = get_default_skip_patterns();
        let hex_pattern = &patterns[1]; // Second pattern should be hex colors

        assert!(hex_pattern.is_match("#deadbeef"));
        assert!(hex_pattern.is_match("#fff"));
        assert!(hex_pattern.is_match("#123456"));
        assert!(!hex_pattern.is_match("deadbeef")); // Without #
        assert!(!hex_pattern.is_match("#gg")); // Invalid hex
    }

    #[test]
    fn test_email_pattern() {
        let patterns = get_default_skip_patterns();
        let email_pattern = &patterns[2]; // Third pattern should be emails

        assert!(email_pattern.is_match("user@example.com"));
        assert!(email_pattern.is_match("test.email+tag@domain.co.uk"));
        assert!(!email_pattern.is_match("not an email"));
    }

    #[test]
    fn test_compile_user_patterns() {
        let user_patterns = vec![
            r"\b[A-Z]{2,}\b".to_string(), // All caps words
            r"TODO:.*".to_string(),       // TODO comments
        ];

        let compiled = compile_user_patterns(&user_patterns).unwrap();
        assert_eq!(compiled.len(), 2);

        assert!(compiled[0].is_match("HTML"));
        assert!(compiled[1].is_match("TODO: fix this"));
    }

    #[test]
    fn test_invalid_user_pattern() {
        let invalid_patterns = vec![r"[invalid".to_string()]; // Missing closing bracket

        assert!(compile_user_patterns(&invalid_patterns).is_err());
    }
}
