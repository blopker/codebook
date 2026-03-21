use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ConfigSettings {
    /// List of dictionaries to use for spell checking
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dictionaries: Vec<String>,

    /// Custom allowlist of words
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<String>,

    /// Words that should always be flagged
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flag_words: Vec<String>,

    /// Glob patterns for paths to include
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_paths: Vec<String>,

    /// Glob patterns for paths to ignore
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_paths: Vec<String>,

    /// Regex patterns for text to ignore
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_patterns: Vec<String>,

    /// Whether to use global configuration
    #[serde(
        default = "default_use_global",
        skip_serializing_if = "is_default_use_global"
    )]
    pub use_global: bool,

    /// Minimum word length to check (words shorter than this are ignored)
    #[serde(
        default = "default_min_word_length",
        skip_serializing_if = "is_default_min_word_length"
    )]
    pub min_word_length: usize,

    /// Tag prefixes to include (if non-empty, only matching tags are checked)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tags: Vec<String>,

    /// Tag prefixes to exclude (takes precedence over include_tags)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_tags: Vec<String>,
}

fn default_use_global() -> bool {
    true
}

fn is_default_use_global(value: &bool) -> bool {
    *value == default_use_global()
}

fn default_min_word_length() -> usize {
    3
}

fn is_default_min_word_length(value: &usize) -> bool {
    *value == default_min_word_length()
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            dictionaries: vec![],
            words: Vec::new(),
            flag_words: Vec::new(),
            include_paths: Vec::new(),
            ignore_paths: Vec::new(),
            ignore_patterns: Vec::new(),
            use_global: true,
            min_word_length: default_min_word_length(),
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
        }
    }
}

impl<'de> Deserialize<'de> for ConfigSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn to_lowercase_vec(v: Vec<String>) -> Vec<String> {
            v.into_iter().map(|s| s.to_ascii_lowercase()).collect()
        }
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            dictionaries: Vec<String>,
            #[serde(default)]
            words: Vec<String>,
            #[serde(default)]
            flag_words: Vec<String>,
            #[serde(default)]
            include_paths: Vec<String>,
            #[serde(default)]
            ignore_paths: Vec<String>,
            #[serde(default)]
            ignore_patterns: Vec<String>,
            #[serde(default = "default_use_global")]
            use_global: bool,
            #[serde(default = "default_min_word_length")]
            min_word_length: usize,
            #[serde(default)]
            include_tags: Vec<String>,
            #[serde(default)]
            exclude_tags: Vec<String>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(ConfigSettings {
            dictionaries: to_lowercase_vec(helper.dictionaries),
            words: to_lowercase_vec(helper.words),
            flag_words: to_lowercase_vec(helper.flag_words),
            include_paths: helper.include_paths,
            ignore_paths: helper.ignore_paths,
            ignore_patterns: helper.ignore_patterns,
            use_global: helper.use_global,
            min_word_length: helper.min_word_length,
            include_tags: helper.include_tags,
            exclude_tags: helper.exclude_tags,
        })
    }
}

impl ConfigSettings {
    /// Merge another config settings into this one, sorting and deduplicating all collections
    pub fn merge(&mut self, other: ConfigSettings) {
        // Add items from the other config
        self.dictionaries.extend(other.dictionaries);
        self.words.extend(other.words);
        self.flag_words.extend(other.flag_words);
        self.include_paths.extend(other.include_paths);
        self.ignore_paths.extend(other.ignore_paths);
        self.ignore_patterns.extend(other.ignore_patterns);
        self.include_tags.extend(other.include_tags);
        self.exclude_tags.extend(other.exclude_tags);

        // The use_global setting from the other config is ignored during merging
        // as this is a per-config setting

        // Override min_word_length if the other config has a non-default value
        if other.min_word_length != default_min_word_length() {
            self.min_word_length = other.min_word_length;
        }

        // Sort and deduplicate each collection
        self.sort_and_dedup();
    }

    /// Sort and deduplicate all collections in the config
    pub fn sort_and_dedup(&mut self) {
        // Sort and deduplicate each Vec
        sort_and_dedup(&mut self.dictionaries);
        sort_and_dedup(&mut self.words);
        sort_and_dedup(&mut self.flag_words);
        sort_and_dedup(&mut self.include_paths);
        sort_and_dedup(&mut self.ignore_paths);
        sort_and_dedup(&mut self.ignore_patterns);
        sort_and_dedup(&mut self.include_tags);
        sort_and_dedup(&mut self.exclude_tags);
    }
}

/// Check if a tag matches a pattern using prefix matching.
/// "comment" matches "comment", "comment.line", "comment.block", etc.
fn tag_matches_pattern(tag: &str, pattern: &str) -> bool {
    tag == pattern || tag.starts_with(pattern) && tag.as_bytes().get(pattern.len()) == Some(&b'.')
}

impl ConfigSettings {
    /// Determine whether a capture tag should be spell-checked based on
    /// include_tags and exclude_tags. exclude_tags takes precedence.
    pub fn should_check_tag(&self, tag: &str) -> bool {
        // exclude_tags takes precedence
        if self
            .exclude_tags
            .iter()
            .any(|p| tag_matches_pattern(tag, p))
        {
            return false;
        }
        // if include_tags is set, tag must match at least one
        if !self.include_tags.is_empty() {
            return self
                .include_tags
                .iter()
                .any(|p| tag_matches_pattern(tag, p));
        }
        true
    }

    /// Insert a word into the allowlist, returning true when it was newly added.
    pub fn insert_word(&mut self, word: &str) -> bool {
        let word = word.to_ascii_lowercase();
        if self.words.contains(&word) {
            return false;
        }
        self.words.push(word);
        self.words.sort();
        self.words.dedup();
        true
    }

    /// Insert a path into the ignore list, returning true when it was newly added.
    pub fn insert_ignore(&mut self, file: &str) -> bool {
        let file = file.to_string();
        if self.ignore_paths.contains(&file) {
            return false;
        }
        self.ignore_paths.push(file);
        self.ignore_paths.sort();
        self.ignore_paths.dedup();
        true
    }

    /// Insert a path into the include list, returning true when it was newly added.
    pub fn insert_include(&mut self, file: &str) -> bool {
        let file = file.to_string();
        if self.include_paths.contains(&file) {
            return false;
        }
        self.include_paths.push(file);
        self.include_paths.sort();
        self.include_paths.dedup();
        true
    }

    /// Resolve configured dictionary IDs, providing a default when none are set.
    pub fn dictionary_ids(&self) -> Vec<String> {
        if self.dictionaries.is_empty() {
            vec!["en_us".to_string()]
        } else {
            self.dictionaries.clone()
        }
    }

    /// Determine whether a path should be included based on the configured glob patterns.
    pub fn should_include_path(&self, path: &Path) -> bool {
        if self.include_paths.is_empty() {
            return true;
        }
        let path_str = path.to_string_lossy();
        match_pattern(&self.include_paths, &path_str)
    }

    /// Determine whether a path should be ignored based on the configured glob patterns.
    pub fn should_ignore_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        match_pattern(&self.ignore_paths, &path_str)
    }

    /// Check if a word is explicitly allowed.
    pub fn is_allowed_word(&self, word: &str) -> bool {
        let word = word.to_ascii_lowercase();
        self.words.iter().any(|w| w == &word)
    }

    /// Check if a word should be flagged.
    pub fn should_flag_word(&self, word: &str) -> bool {
        let word = word.to_ascii_lowercase();
        self.flag_words.iter().any(|w| w == &word)
    }

    /// Retrieve the configured minimum word length.
    pub fn min_word_length(&self) -> usize {
        self.min_word_length
    }
}

fn match_pattern(patterns: &[String], path_str: &str) -> bool {
    patterns.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches(path_str))
            .unwrap_or(false)
    })
}

/// Helper function to sort and deduplicate a Vec of strings
fn sort_and_dedup(vec: &mut Vec<String>) {
    vec.sort();
    vec.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let config = ConfigSettings::default();
        assert_eq!(config.dictionaries, Vec::<String>::new());
        assert_eq!(config.words, Vec::<String>::new());
        assert_eq!(config.flag_words, Vec::<String>::new());
        assert_eq!(config.include_paths, Vec::<String>::new());
        assert_eq!(config.ignore_paths, Vec::<String>::new());
        assert_eq!(config.ignore_patterns, Vec::<String>::new());
        assert!(config.use_global);
        assert_eq!(config.min_word_length, 3);
    }

    #[test]
    fn test_deserialization() {
        let toml_str = r#"
        dictionaries = ["EN_US", "en_GB"]
        words = ["CodeBook", "Rust"]
        flag_words = ["TODO", "FIXME"]
        include_paths = ["src/**/*.rs", "lib/"]
        ignore_paths = ["**/*.md", "target/"]
        ignore_patterns = ["^```.*$", "^//.*$"]
        use_global = false
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();

        assert_eq!(config.dictionaries, vec!["en_us", "en_gb"]);
        assert_eq!(config.words, vec!["codebook", "rust"]);
        assert_eq!(config.flag_words, vec!["todo", "fixme"]);
        assert_eq!(config.include_paths, vec!["src/**/*.rs", "lib/"]);
        assert_eq!(config.ignore_paths, vec!["**/*.md", "target/"]);

        // Don't test the exact order, just check that both elements are present
        assert_eq!(config.ignore_patterns.len(), 2);
        assert!(config.ignore_patterns.contains(&"^```.*$".to_string()));
        assert!(config.ignore_patterns.contains(&"^//.*$".to_string()));

        assert!(!config.use_global);
    }

    #[test]
    fn test_min_word_length_deserialization() {
        // Test with explicit value
        let toml_str = r#"
        min_word_length = 2
        "#;
        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert_eq!(config.min_word_length, 2);

        // Test with default value (when not specified)
        let toml_str = r#"
        dictionaries = ["en_us"]
        "#;
        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert_eq!(config.min_word_length, 3);
    }

    #[test]
    fn test_serialization() {
        let config = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            words: vec!["rust".to_string()],
            ..Default::default()
        };

        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("dictionaries = [\"en_us\"]"));
        assert!(serialized.contains("words = [\"rust\"]"));
        // Defaults should not be there
        assert!(!serialized.contains("use_global = true"));
        assert!(!serialized.contains("min_word_length = 3"));
    }

    #[test]
    fn test_serialization_roundtrip_include_paths() {
        let config = ConfigSettings {
            include_paths: vec!["src/**/*.rs".to_string(), "lib/".to_string()],
            ..Default::default()
        };

        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("include_paths"));

        let deserialized: ConfigSettings = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.include_paths, vec!["src/**/*.rs", "lib/"]);
    }

    #[test]
    fn test_merge() {
        let mut base = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            words: vec!["codebook".to_string()],
            flag_words: vec!["todo".to_string()],
            include_paths: vec!["src/".to_string()],
            ignore_paths: vec!["**/*.md".to_string()],
            ignore_patterns: vec!["^```.*$".to_string()],
            use_global: true,
            min_word_length: 3,
            ..Default::default()
        };

        let other = ConfigSettings {
            dictionaries: vec!["en_gb".to_string(), "en_us".to_string()],
            words: vec!["rust".to_string()],
            flag_words: vec!["fixme".to_string()],
            include_paths: vec!["lib/".to_string(), "src/".to_string()],
            ignore_paths: vec!["target/".to_string()],
            ignore_patterns: vec!["^//.*$".to_string()],
            use_global: false,
            min_word_length: 2,
            ..Default::default()
        };

        base.merge(other);

        // After merging and deduplicating, we should have combined items
        assert_eq!(base.dictionaries, vec!["en_gb", "en_us"]);
        assert_eq!(base.words, vec!["codebook", "rust"]);
        assert_eq!(base.flag_words, vec!["fixme", "todo"]);
        assert_eq!(base.include_paths, vec!["lib/", "src/"]);
        assert_eq!(base.ignore_paths, vec!["**/*.md", "target/"]);

        // Don't test the exact order, just check that both elements are present
        assert_eq!(base.ignore_patterns.len(), 2);
        assert!(base.ignore_patterns.contains(&"^```.*$".to_string()));
        assert!(base.ignore_patterns.contains(&"^//.*$".to_string()));

        // use_global from the base should be preserved
        assert!(base.use_global);
        // min_word_length from other should override base (since it's non-default)
        assert_eq!(base.min_word_length, 2);
    }

    #[test]
    fn test_merge_min_word_length_default() {
        let mut base = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            min_word_length: 5,
            ..Default::default()
        };

        let other = ConfigSettings {
            dictionaries: vec!["en_gb".to_string()],
            min_word_length: 3, // default value
            ..Default::default()
        };

        base.merge(other);

        // min_word_length from base should be preserved when other has default
        assert_eq!(base.min_word_length, 5);
    }

    #[test]
    fn test_sort_and_dedup() {
        let mut config = ConfigSettings {
            dictionaries: vec![
                "en_gb".to_string(),
                "en_us".to_string(),
                "en_gb".to_string(),
            ],
            words: vec![
                "rust".to_string(),
                "codebook".to_string(),
                "rust".to_string(),
            ],
            flag_words: vec!["fixme".to_string(), "todo".to_string(), "fixme".to_string()],
            include_paths: vec![],
            ignore_paths: vec![
                "target/".to_string(),
                "**/*.md".to_string(),
                "target/".to_string(),
            ],
            ignore_patterns: vec![
                "^//.*$".to_string(),
                "^```.*$".to_string(),
                "^//.*$".to_string(),
            ],
            use_global: true,
            min_word_length: 3,
            ..Default::default()
        };

        config.sort_and_dedup();

        assert_eq!(config.dictionaries, vec!["en_gb", "en_us"]);
        assert_eq!(config.words, vec!["codebook", "rust"]);
        assert_eq!(config.flag_words, vec!["fixme", "todo"]);
        assert_eq!(config.ignore_paths, vec!["**/*.md", "target/"]);

        // Don't test the exact order, just check that both elements are present and duplicates removed
        assert_eq!(config.ignore_patterns.len(), 2);
        assert!(config.ignore_patterns.contains(&"^```.*$".to_string()));
        assert!(config.ignore_patterns.contains(&"^//.*$".to_string()));
    }

    #[test]
    fn test_use_global_default() {
        let toml_str = r#"
        dictionaries = ["EN_US"]
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert!(config.use_global);
    }

    #[test]
    fn test_empty_deserialization() {
        let toml_str = "";
        let config: ConfigSettings = toml::from_str(toml_str).unwrap();

        assert_eq!(config, ConfigSettings::default());
    }

    #[test]
    fn test_include_tags_deserialization() {
        let toml_str = r#"
        include_tags = ["comment", "string"]
        "#;
        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert_eq!(config.include_tags, vec!["comment", "string"]);
        assert!(config.exclude_tags.is_empty());
    }

    #[test]
    fn test_exclude_tags_deserialization() {
        let toml_str = r#"
        exclude_tags = ["identifier.variable", "identifier.parameter"]
        "#;
        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert!(config.include_tags.is_empty());
        assert_eq!(
            config.exclude_tags,
            vec!["identifier.variable", "identifier.parameter"]
        );
    }

    #[test]
    fn test_tags_default_empty() {
        let config = ConfigSettings::default();
        assert!(config.include_tags.is_empty());
        assert!(config.exclude_tags.is_empty());
    }

    #[test]
    fn test_tags_serialization_omitted_when_empty() {
        let config = ConfigSettings::default();
        let serialized = toml::to_string(&config).unwrap();
        assert!(!serialized.contains("include_tags"));
        assert!(!serialized.contains("exclude_tags"));
    }

    #[test]
    fn test_tags_serialization_present_when_set() {
        let config = ConfigSettings {
            include_tags: vec!["comment".to_string()],
            ..Default::default()
        };
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("include_tags"));
    }

    #[test]
    fn test_tags_merge() {
        let mut base = ConfigSettings {
            include_tags: vec!["comment".to_string()],
            exclude_tags: vec!["identifier.type".to_string()],
            ..Default::default()
        };
        let other = ConfigSettings {
            include_tags: vec!["string".to_string(), "comment".to_string()],
            exclude_tags: vec!["identifier.module".to_string()],
            ..Default::default()
        };
        base.merge(other);
        assert_eq!(base.include_tags, vec!["comment", "string"]);
        assert_eq!(
            base.exclude_tags,
            vec!["identifier.module", "identifier.type"]
        );
    }

    #[test]
    fn test_should_check_tag_no_filters() {
        let config = ConfigSettings::default();
        assert!(config.should_check_tag("comment"));
        assert!(config.should_check_tag("string"));
        assert!(config.should_check_tag("identifier.function"));
    }

    #[test]
    fn test_should_check_tag_include_only() {
        let config = ConfigSettings {
            include_tags: vec!["comment".to_string(), "string".to_string()],
            ..Default::default()
        };
        assert!(config.should_check_tag("comment"));
        assert!(config.should_check_tag("comment.line"));
        assert!(config.should_check_tag("comment.block"));
        assert!(config.should_check_tag("string"));
        assert!(config.should_check_tag("string.special"));
        assert!(!config.should_check_tag("identifier"));
        assert!(!config.should_check_tag("identifier.function"));
    }

    #[test]
    fn test_should_check_tag_exclude_only() {
        let config = ConfigSettings {
            exclude_tags: vec!["identifier.variable".to_string()],
            ..Default::default()
        };
        assert!(config.should_check_tag("comment"));
        assert!(config.should_check_tag("identifier.function"));
        assert!(!config.should_check_tag("identifier.variable"));
    }

    #[test]
    fn test_should_check_tag_both_include_and_exclude() {
        // include comments and strings, but exclude string.heredoc
        let config = ConfigSettings {
            include_tags: vec!["comment".to_string(), "string".to_string()],
            exclude_tags: vec!["string.heredoc".to_string()],
            ..Default::default()
        };
        assert!(config.should_check_tag("comment"));
        assert!(config.should_check_tag("comment.line"));
        assert!(config.should_check_tag("string"));
        assert!(config.should_check_tag("string.special"));
        assert!(!config.should_check_tag("string.heredoc"));
        assert!(!config.should_check_tag("identifier.function"));
    }

    #[test]
    fn test_should_check_tag_exclude_prefix() {
        // Excluding "identifier" should exclude all identifier sub-tags
        let config = ConfigSettings {
            exclude_tags: vec!["identifier".to_string()],
            ..Default::default()
        };
        assert!(config.should_check_tag("comment"));
        assert!(config.should_check_tag("string"));
        assert!(!config.should_check_tag("identifier"));
        assert!(!config.should_check_tag("identifier.function"));
        assert!(!config.should_check_tag("identifier.type"));
    }

    #[test]
    fn test_partial_deserialization() {
        let toml_str = r#"
        dictionaries = ["EN_US"]
        words = ["CodeBook"]
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();

        assert_eq!(config.dictionaries, vec!["en_us"]);
        assert_eq!(config.words, vec!["codebook"]);
        assert_eq!(config.flag_words, Vec::<String>::new());
        assert_eq!(config.ignore_paths, Vec::<String>::new());
        assert_eq!(config.ignore_patterns, Vec::<String>::new());
        assert!(config.use_global);
    }
}
