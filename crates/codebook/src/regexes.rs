use regex::Regex;
use std::sync::LazyLock;

/// Git commit hashes: a hex run of 7-40 chars containing at least one digit.
/// The digit requirement keeps real words spelled with only a-f letters
/// ("acceded", "defaced" — and misspellings of them) checkable. The regex
/// crate has no lookahead, so enumerate the first digit's position: either
/// it's within the first 7 chars (exact 7-40 length bounds per position),
/// or the token starts with 7+ letters and a digit follows.
fn git_hash_pattern() -> Regex {
    let alts: Vec<String> = (0..7)
        .map(|i| format!("[a-fA-F]{{{i}}}[0-9][0-9a-fA-F]{{{},{}}}", 6 - i, 39 - i))
        .chain(std::iter::once(
            "[a-fA-F]{7,39}[0-9][0-9a-fA-F]{0,32}".to_string(),
        ))
        .collect();
    Regex::new(&format!(r"\b(?:{})\b", alts.join("|"))).expect("Valid git hash regex")
}

static DEFAULT_SKIP_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // URLs (http/https)
        Regex::new(r"https?://[^\s]+").expect("Valid URL regex"),
        // Hex colors (#deadbeef, #fff, #123456)
        Regex::new(r"#[0-9a-fA-F]{3,8}").expect("Valid hex color regex"),
        // Email addresses
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").expect("Valid email regex"),
        // File paths (Unix-style, absolute or ./-relative). Anchored to line
        // start or a preceding separator so a slash inside a word ("and/or")
        // doesn't swallow the following word.
        Regex::new(r#"(?m)(?:^|[\s"'(\[=<,])\.{0,2}/[^\s]*"#).expect("Valid Unix path regex"),
        // File paths (Windows-style with drive letter)
        Regex::new(r"[A-Za-z]:\\[^\s]*").expect("Valid Windows path regex"),
        // UUID
        Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
            .expect("Valid UUID regex"),
        // Base64 strings (requires trailing = padding to avoid false positives)
        Regex::new(r"[A-Za-z0-9+/]{20,}={1,2}").expect("Valid Base64 regex"),
        // Git commit hashes (7+ hex characters, at least one digit)
        git_hash_pattern(),
        // Markdown/HTML links (URL part must not contain spaces)
        Regex::new(r"\[([^\]]+)\]\([^\s)]+\)").expect("Valid markdown link regex"),
        // Quoted paths/URIs: a space-free quoted string containing a slash,
        // like import specifiers ('package:flutter/material.dart',
        // "github.com/user/repo"). The slash mid-token can't be told apart
        // from prose like and/or by shape alone; the quotes provide the
        // context that makes it a path.
        Regex::new(r#""[^"\s]*/[^"\s]*"|'[^'\s]*/[^'\s]*'"#).expect("Valid quoted path regex"),
    ]
});

/// Default regex patterns to skip during spell checking.
/// These patterns match common technical strings that contain letter sequences
/// but shouldn't be treated as words for spell checking purposes.
pub fn get_default_skip_patterns() -> &'static Vec<Regex> {
    &DEFAULT_SKIP_PATTERNS
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
    fn test_base64_pattern() {
        let patterns = get_default_skip_patterns();
        let base64_pattern = &patterns[6]; // Base64 pattern

        // Real base64 strings (with padding)
        assert!(base64_pattern.is_match("dGVzdCBiYXNlNjQgZW5jb2Rpbmc=")); // "test base64 encoding"
        assert!(base64_pattern.is_match("SGVsbG8gV29ybGQhIFRoaXMgaXM=")); // long enough base64
        // No padding = no match
        assert!(!base64_pattern.is_match("dGVzdCBiYXNlNjQgZW5jb2Rpbmc"));

        // Path-like strings should NOT match
        assert!(!base64_pattern.is_match("administraton/dashboard"));
        assert!(!base64_pattern.is_match("some/long/path/to/a/file"));
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
    fn test_unix_path_pattern() {
        let patterns = get_default_skip_patterns();
        let path_pattern = &patterns[3];

        // Paths at line start, after whitespace, and in common wrappers
        assert!(path_pattern.is_match("/usr/local/bin"));
        assert!(path_pattern.is_match("see /tmp/file.txt for details"));
        assert!(path_pattern.is_match("path=\"/etc/hosts\""));
        assert!(path_pattern.is_match("first line\n/var/log/syslog"));
        assert!(path_pattern.is_match("import x from './modulle'"));
        assert!(path_pattern.is_match("see ../docs/README"));

        // A slash inside a word must not start a "path"
        assert!(!path_pattern.is_match("and/or"));
        assert!(!path_pattern.is_match("either/neither"));
    }

    #[test]
    fn test_quoted_path_pattern() {
        let patterns = get_default_skip_patterns();
        let quoted_pattern = &patterns[9];

        // Import-style URIs and quoted paths
        assert!(quoted_pattern.is_match("import 'package:flutter/material.dart';"));
        assert!(quoted_pattern.is_match(r#"import "github.com/user/repo""#));
        assert!(quoted_pattern.is_match("require('./lib/utils')"));

        // Quoted prose with spaces is not a path, and bare words don't match
        assert!(!quoted_pattern.is_match("'choose one and/or both'"));
        assert!(!quoted_pattern.is_match("'hello'"));
        assert!(!quoted_pattern.is_match("and/or"));
    }

    #[test]
    fn test_git_hash_pattern() {
        let patterns = get_default_skip_patterns();
        let hash_pattern = &patterns[7];

        // Realistic hashes (contain digits) match at any digit position
        assert!(hash_pattern.is_match("babc157"));
        assert!(hash_pattern.is_match("abc1def")); // digit mid-token
        assert!(hash_pattern.is_match("deadbeef0")); // digit last
        assert!(hash_pattern.is_match("1234567")); // digits only
        assert!(hash_pattern.is_match("d670460b4b4aece5915caf5c68d12f560a9fe3e4"));

        // Words spelled with only a-f letters are not hashes
        assert!(!hash_pattern.is_match("acceded"));
        assert!(!hash_pattern.is_match("defaced"));
        assert!(!hash_pattern.is_match("accedded")); // misspelling stays visible
        assert!(!hash_pattern.is_match("deadbeef")); // no digit

        // Too short
        assert!(!hash_pattern.is_match("abc123"));
    }
}
