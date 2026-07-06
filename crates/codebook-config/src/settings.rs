use glob::Pattern;
use log::warn;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single `[[overrides]]` block in the config file.
///
/// Word-related fields deserialize through `lowercase_*` so lookups are
/// case-insensitive; paths and regex patterns keep their original casing.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct OverrideBlock {
    /// Required: glob patterns matched against file path relative to project root.
    /// Defaulted rather than required so a block missing `paths` is filtered out
    /// with a warning instead of failing the whole config parse.
    #[serde(default)]
    pub paths: Vec<String>,

    // --- Replace fields (replace the base list entirely) ---
    #[serde(
        default,
        deserialize_with = "lowercase_opt_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub dictionaries: Option<Vec<String>>,

    #[serde(
        default,
        deserialize_with = "lowercase_opt_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub words: Option<Vec<String>>,

    #[serde(
        default,
        deserialize_with = "lowercase_opt_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub flag_words: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,

    // --- Append fields (append to the resolved list) ---
    #[serde(
        default,
        deserialize_with = "lowercase_opt_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub extra_dictionaries: Option<Vec<String>>,

    #[serde(
        default,
        deserialize_with = "lowercase_opt_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub extra_words: Option<Vec<String>>,

    #[serde(
        default,
        deserialize_with = "lowercase_opt_vec",
        skip_serializing_if = "Option::is_none"
    )]
    pub extra_flag_words: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_ignore_patterns: Option<Vec<String>>,
}

fn lowercase_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Vec::<String>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|s| s.to_ascii_lowercase()).collect())
}

fn lowercase_opt_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(v.map(|v| v.into_iter().map(|s| s.to_ascii_lowercase()).collect()))
}

/// Deserialize override blocks, dropping invalid and no-op ones with a warning.
fn valid_overrides<'de, D>(deserializer: D) -> Result<Vec<OverrideBlock>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let blocks = Vec::<OverrideBlock>::deserialize(deserializer)?;
    Ok(blocks
        .into_iter()
        .filter(|o| {
            if !o.is_valid() {
                warn!("Skipping invalid override block (empty or invalid paths)");
                return false;
            }
            if !o.has_effect() {
                warn!("Skipping no-op override block (no settings specified)");
                return false;
            }
            true
        })
        .collect())
}

impl OverrideBlock {
    /// Returns true if this override block is valid (has non-empty paths with at least one valid glob).
    pub fn is_valid(&self) -> bool {
        if self.paths.is_empty() {
            return false;
        }
        self.paths.iter().any(|p| Pattern::new(p).is_ok())
    }

    /// Check if this override applies to the given relative file path.
    pub fn matches_path(&self, relative_path: &Path) -> bool {
        let path_str = normalize_separators(&relative_path.to_string_lossy());
        self.paths.iter().any(|pattern| {
            Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
        })
    }

    /// Returns true if any field besides `paths` is set (the override has an effect).
    pub fn has_effect(&self) -> bool {
        self.dictionaries.is_some()
            || self.words.is_some()
            || self.flag_words.is_some()
            || self.ignore_patterns.is_some()
            || self.extra_dictionaries.is_some()
            || self.extra_words.is_some()
            || self.extra_flag_words.is_some()
            || self.extra_ignore_patterns.is_some()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConfigSettings {
    /// List of dictionaries to use for spell checking.
    /// Dictionary IDs are language codes (e.g. "en_US") — normalized to
    /// lowercase so lookups are case-insensitive. Word lists keep their
    /// original casing and are compared via unicase::eq.
    #[serde(
        default,
        deserialize_with = "lowercase_vec",
        skip_serializing_if = "Vec::is_empty"
    )]
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

    /// Minimum word length to check (words shorter than this are ignored).
    /// None means "not set" so merging can tell an explicit value — even one
    /// equal to the default — apart from an omitted one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_word_length: Option<usize>,

    /// Tag prefixes to include (if non-empty, only matching tags are checked)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tags: Vec<String>,

    /// Tag prefixes to exclude (takes precedence over include_tags)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_tags: Vec<String>,

    /// Scoped configuration overrides
    #[serde(
        default,
        deserialize_with = "valid_overrides",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub overrides: Vec<OverrideBlock>,
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
            min_word_length: None,
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
            overrides: Vec::new(),
        }
    }
}

impl ConfigSettings {
    /// Merge another config settings into this one, sorting and deduplicating all collections.
    /// Overrides are appended (preserving order: self's overrides first, then other's).
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

        // Append overrides (global first, then project — order matters)
        self.overrides.extend(other.overrides);

        // The use_global setting from the other config is ignored during merging
        // as this is a per-config setting

        // Override min_word_length if the other config sets one explicitly
        if other.min_word_length.is_some() {
            self.min_word_length = other.min_word_length;
        }

        // Sort and deduplicate each collection (but NOT overrides)
        self.sort_and_dedup();
    }

    /// Sort and deduplicate all collections in the config (but not overrides).
    pub fn sort_and_dedup(&mut self) {
        // Sort and deduplicate each Vec. Word lists dedup case-insensitively
        // to match how lookups compare them (unicase::eq).
        sort_and_dedup(&mut self.dictionaries);
        sort_and_dedup_unicase(&mut self.words);
        sort_and_dedup_unicase(&mut self.flag_words);
        sort_and_dedup(&mut self.include_paths);
        sort_and_dedup(&mut self.ignore_paths);
        sort_and_dedup(&mut self.ignore_patterns);
        sort_and_dedup(&mut self.include_tags);
        sort_and_dedup(&mut self.exclude_tags);
        // Note: overrides are NOT sorted — order matters for resolution
    }

    /// Apply a single override block to this settings (mutates in place).
    /// Replace fields are applied first, then append fields.
    pub fn apply_override(&mut self, over: &OverrideBlock) {
        // Replace fields: fully replace the list
        if let Some(ref v) = over.dictionaries {
            self.dictionaries = v.clone();
        }
        if let Some(ref v) = over.words {
            self.words = v.clone();
        }
        if let Some(ref v) = over.flag_words {
            self.flag_words = v.clone();
        }
        if let Some(ref v) = over.ignore_patterns {
            self.ignore_patterns = v.clone();
        }

        // Append fields: extend the current list
        if let Some(ref v) = over.extra_dictionaries {
            self.dictionaries.extend(v.clone());
        }
        if let Some(ref v) = over.extra_words {
            self.words.extend(v.clone());
        }
        if let Some(ref v) = over.extra_flag_words {
            self.flag_words.extend(v.clone());
        }
        if let Some(ref v) = over.extra_ignore_patterns {
            self.ignore_patterns.extend(v.clone());
        }
    }

    /// Resolve the effective settings for a specific file path by applying matching overrides.
    /// Returns a new ConfigSettings with overrides applied and the overrides list cleared.
    pub fn resolve_for_path(&self, path: &Path) -> ConfigSettings {
        let mut resolved = self.clone();
        resolved.overrides = vec![]; // Resolved config has no overrides

        for ovr in &self.overrides {
            if ovr.matches_path(path) {
                resolved.apply_override(ovr);
            }
        }

        resolved
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
    /// Existing entries differing only in case are treated as duplicates.
    pub fn insert_word(&mut self, word: &str) -> bool {
        if self.words.iter().any(|w| unicase::eq(w.as_str(), word)) {
            return false;
        }
        self.words.push(word.to_string());
        self.words.sort();
        true
    }

    /// Insert a path into the ignore list, returning true when it was newly added.
    pub fn insert_ignore(&mut self, file: &str) -> bool {
        let file = normalize_separators(file);
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
        let file = normalize_separators(file);
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
        let path_str = normalize_separators(&path.to_string_lossy());
        match_pattern(&self.include_paths, &path_str)
    }

    /// Determine whether a path should be ignored based on the configured glob patterns.
    pub fn should_ignore_path(&self, path: &Path) -> bool {
        let path_str = normalize_separators(&path.to_string_lossy());
        match_pattern(&self.ignore_paths, &path_str)
    }

    /// Check if a word is explicitly allowed.
    pub fn is_allowed_word(&self, word: &str) -> bool {
        self.words.iter().any(|w| unicase::eq(w.as_str(), word))
    }

    /// Check if a word should be flagged.
    pub fn should_flag_word(&self, word: &str) -> bool {
        self.flag_words
            .iter()
            .any(|w| unicase::eq(w.as_str(), word))
    }

    /// Retrieve the configured minimum word length.
    pub fn min_word_length(&self) -> usize {
        self.min_word_length.unwrap_or_else(default_min_word_length)
    }
}

fn match_pattern(patterns: &[String], path_str: &str) -> bool {
    patterns.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches(path_str))
            .unwrap_or(false)
    })
}

/// Config globs always use `/` as the separator, like `.gitignore`, so on
/// Windows native `\` separators are normalized to `/`, both for paths being
/// matched and for paths stored as glob entries. Stored entries especially
/// must use `/` to stay portable: in a pattern `\` is a literal character on
/// Unix, so an entry written with backslashes on Windows would silently stop
/// matching when the config is shared to WSL/macOS/Linux.
/// On Unix `\` is an ordinary filename character and paths are left alone.
fn normalize_separators(path_str: &str) -> String {
    if cfg!(windows) {
        path_str.replace('\\', "/")
    } else {
        path_str.to_string()
    }
}

/// Helper function to sort and deduplicate a Vec of strings
fn sort_and_dedup(vec: &mut Vec<String>) {
    vec.sort();
    vec.dedup();
}

/// Sort and deduplicate a word list, treating entries that differ only in case
/// as duplicates (the first occurrence in sort order wins).
fn sort_and_dedup_unicase(vec: &mut Vec<String>) {
    vec.sort_by(|a, b| {
        unicase::UniCase::new(a.as_str())
            .cmp(&unicase::UniCase::new(b.as_str()))
            .then_with(|| a.cmp(b))
    });
    vec.dedup_by(|a, b| unicase::eq(a.as_str(), b.as_str()));
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
        assert_eq!(config.min_word_length, None);
        assert_eq!(config.min_word_length(), 3);
        assert!(config.overrides.is_empty());
    }

    #[test]
    fn test_deserialization() {
        let toml_str = r#"
        dictionaries = ["EN_US", "en_GB"]
        words = ["CodeBook", "Rust", "Апгрейдить"]
        flag_words = ["TODO", "FIXME", "Ошибка"]
        include_paths = ["src/**/*.rs", "lib/"]
        ignore_paths = ["**/*.md", "target/"]
        ignore_patterns = ["^```.*$", "^//.*$"]
        use_global = false
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();

        assert_eq!(config.dictionaries, vec!["en_us", "en_gb"]);
        assert_eq!(config.words, vec!["CodeBook", "Rust", "Апгрейдить"]);
        assert_eq!(config.flag_words, vec!["TODO", "FIXME", "Ошибка"]);
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
        assert_eq!(config.min_word_length, Some(2));
        assert_eq!(config.min_word_length(), 2);

        // Test with default value (when not specified)
        let toml_str = r#"
        dictionaries = ["en_us"]
        "#;
        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert_eq!(config.min_word_length, None);
        assert_eq!(config.min_word_length(), 3);
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
            min_word_length: Some(3),
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
            min_word_length: Some(2),
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
        // min_word_length from other should override base (since it's explicitly set)
        assert_eq!(base.min_word_length, Some(2));
    }

    #[test]
    fn test_merge_min_word_length_default() {
        let mut base = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            min_word_length: Some(5),
            ..Default::default()
        };

        let other = ConfigSettings {
            dictionaries: vec!["en_gb".to_string()],
            min_word_length: None, // not set
            ..Default::default()
        };

        base.merge(other);

        // min_word_length from base should be preserved when other doesn't set it
        assert_eq!(base.min_word_length, Some(5));
    }

    #[test]
    fn test_merge_min_word_length_explicit_default_wins() {
        let mut base = ConfigSettings {
            min_word_length: Some(5),
            ..Default::default()
        };

        let other = ConfigSettings {
            // Explicitly set to the default value — should still override
            min_word_length: Some(3),
            ..Default::default()
        };

        base.merge(other);

        assert_eq!(base.min_word_length, Some(3));
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
            min_word_length: Some(3),
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
    fn test_unicode_words_ignore_case() {
        let mut config = ConfigSettings::default();

        assert!(config.insert_word("Апгрейдить"));
        assert!(!config.insert_word("апгрейдить"));
        assert_eq!(config.words, vec!["Апгрейдить"]);
        assert!(config.is_allowed_word("АПГРЕЙДИТЬ"));
        assert!(config.is_allowed_word("апгрейдить"));

        config.flag_words.push("ошибка".to_string());
        assert!(config.should_flag_word("Ошибка"));
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
        assert_eq!(config.words, vec!["CodeBook"]);
        assert_eq!(config.flag_words, Vec::<String>::new());
        assert_eq!(config.ignore_paths, Vec::<String>::new());
        assert_eq!(config.ignore_patterns, Vec::<String>::new());
        assert!(config.use_global);
    }

    // --- Override tests ---

    #[test]
    fn test_override_block_deserialization() {
        let toml_str = r#"
        words = ["base"]

        [[overrides]]
        paths = ["**/*.md"]
        extra_words = ["Markdown"]
        dictionaries = ["EN_GB"]
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert_eq!(config.overrides.len(), 1);
        let ovr = &config.overrides[0];
        assert_eq!(ovr.paths, vec!["**/*.md"]);
        assert_eq!(ovr.extra_words, Some(vec!["markdown".to_string()])); // lowercased
        assert_eq!(ovr.dictionaries, Some(vec!["en_gb".to_string()])); // lowercased
        assert_eq!(ovr.words, None);
        assert_eq!(ovr.ignore_patterns, None);
    }

    #[test]
    fn test_override_block_empty_paths_skipped() {
        let toml_str = r#"
        [[overrides]]
        paths = []
        extra_words = ["test"]
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert!(config.overrides.is_empty());
    }

    #[test]
    fn test_override_block_no_effect_skipped() {
        let toml_str = r#"
        [[overrides]]
        paths = ["**/*.md"]
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();
        assert!(config.overrides.is_empty());
    }

    #[test]
    fn test_override_matches_path() {
        let ovr = OverrideBlock {
            paths: vec!["**/*.md".to_string(), "docs/**/*".to_string()],
            extra_words: Some(vec!["test".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        assert!(ovr.matches_path(Path::new("README.md")));
        assert!(ovr.matches_path(Path::new("src/guide.md")));
        assert!(ovr.matches_path(Path::new("docs/api/index.html")));
        assert!(!ovr.matches_path(Path::new("src/main.rs")));
    }

    #[cfg(windows)]
    #[test]
    fn test_override_matches_backslash_path_on_windows() {
        let ovr = OverrideBlock {
            paths: vec!["docs/**/*".to_string()],
            extra_words: Some(vec!["test".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        assert!(ovr.matches_path(Path::new(r"docs\api\guide.md")));
    }

    #[cfg(windows)]
    #[test]
    fn test_ignore_and_include_match_backslash_paths_on_windows() {
        let settings = ConfigSettings {
            include_paths: vec!["src/**/*.rs".to_string()],
            ignore_paths: vec!["target/**/*".to_string()],
            ..Default::default()
        };

        assert!(settings.should_include_path(Path::new(r"src\main.rs")));
        assert!(settings.should_ignore_path(Path::new(r"target\debug\build")));
    }

    #[cfg(windows)]
    #[test]
    fn test_insert_ignore_normalizes_separators_on_windows() {
        let mut settings = ConfigSettings::default();

        // Stored entries become glob patterns and must use `/` so they stay
        // portable when the config is shared with Unix platforms.
        assert!(settings.insert_ignore(r"src\notes.md"));
        assert_eq!(settings.ignore_paths, vec!["src/notes.md"]);
        assert!(settings.should_ignore_path(Path::new(r"src\notes.md")));
    }

    #[cfg(not(windows))]
    #[test]
    fn test_backslash_is_not_a_separator_on_unix() {
        let mut settings = ConfigSettings::default();

        // On Unix `\` is an ordinary filename character and is preserved.
        assert!(settings.insert_ignore(r"weird\name.md"));
        assert_eq!(settings.ignore_paths, vec![r"weird\name.md"]);
    }

    #[test]
    fn test_apply_override_replace() {
        let mut settings = ConfigSettings {
            words: vec!["alpha".to_string(), "beta".to_string()],
            ..Default::default()
        };

        let over = OverrideBlock {
            paths: vec!["**/*.md".to_string()],
            words: Some(vec!["gamma".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        settings.apply_override(&over);
        assert_eq!(settings.words, vec!["gamma"]);
    }

    #[test]
    fn test_apply_override_append() {
        let mut settings = ConfigSettings {
            words: vec!["alpha".to_string(), "beta".to_string()],
            ..Default::default()
        };

        let ovr = OverrideBlock {
            paths: vec!["**/*.md".to_string()],
            extra_words: Some(vec!["gamma".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        settings.apply_override(&ovr);
        assert_eq!(settings.words, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_apply_override_replace_then_append() {
        let mut settings = ConfigSettings {
            words: vec!["alpha".to_string(), "beta".to_string()],
            ..Default::default()
        };

        let ovr = OverrideBlock {
            paths: vec!["**/*.md".to_string()],
            words: Some(vec!["gamma".to_string()]),
            extra_words: Some(vec!["delta".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        settings.apply_override(&ovr);
        assert_eq!(settings.words, vec!["gamma", "delta"]);
    }

    #[test]
    fn test_apply_override_no_change() {
        let mut settings = ConfigSettings {
            words: vec!["alpha".to_string()],
            dictionaries: vec!["en_us".to_string()],
            ..Default::default()
        };

        let over = OverrideBlock {
            paths: vec!["**/*.md".to_string()],
            extra_flag_words: Some(vec!["hack".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        settings.apply_override(&over);
        // words and dictionaries unchanged
        assert_eq!(settings.words, vec!["alpha"]);
        assert_eq!(settings.dictionaries, vec!["en_us"]);
        // flag_words changed
        assert_eq!(settings.flag_words, vec!["hack"]);
    }

    #[test]
    fn test_resolve_for_path_no_match() {
        let settings = ConfigSettings {
            words: vec!["base".to_string()],
            overrides: vec![OverrideBlock {
                paths: vec!["**/*.md".to_string()],
                extra_words: Some(vec!["markdown".to_string()]),
                ..OverrideBlock::default_for_test()
            }],
            ..Default::default()
        };

        let resolved = settings.resolve_for_path(Path::new("src/main.rs"));
        assert_eq!(resolved.words, vec!["base"]);
        assert!(resolved.overrides.is_empty());
    }

    #[test]
    fn test_resolve_for_path_single_match() {
        let settings = ConfigSettings {
            words: vec!["base".to_string()],
            overrides: vec![OverrideBlock {
                paths: vec!["**/*.md".to_string()],
                extra_words: Some(vec!["markdown".to_string()]),
                ..OverrideBlock::default_for_test()
            }],
            ..Default::default()
        };

        let resolved = settings.resolve_for_path(Path::new("README.md"));
        assert_eq!(resolved.words, vec!["base", "markdown"]);
        assert!(resolved.overrides.is_empty());
    }

    #[test]
    fn test_resolve_for_path_multiple_matches() {
        let settings = ConfigSettings {
            words: vec!["base".to_string()],
            overrides: vec![
                OverrideBlock {
                    paths: vec!["**/*.md".to_string()],
                    extra_words: Some(vec!["markdown".to_string()]),
                    ..OverrideBlock::default_for_test()
                },
                OverrideBlock {
                    paths: vec!["docs/**/*".to_string()],
                    extra_words: Some(vec!["documentation".to_string()]),
                    ..OverrideBlock::default_for_test()
                },
            ],
            ..Default::default()
        };

        let resolved = settings.resolve_for_path(Path::new("docs/guide.md"));
        assert_eq!(resolved.words, vec!["base", "markdown", "documentation"]);
    }

    #[test]
    fn test_resolve_for_path_replace_overrides_base() {
        let settings = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            overrides: vec![OverrideBlock {
                paths: vec!["docs/de/**/*".to_string()],
                dictionaries: Some(vec!["de".to_string()]),
                extra_words: Some(vec!["codebook".to_string()]),
                ..OverrideBlock::default_for_test()
            }],
            ..Default::default()
        };

        let resolved = settings.resolve_for_path(Path::new("docs/de/guide.md"));
        assert_eq!(resolved.dictionaries, vec!["de"]);
        assert_eq!(resolved.words, vec!["codebook"]);
    }

    #[test]
    fn test_merge_preserves_override_order() {
        let mut global = ConfigSettings {
            words: vec!["global".to_string()],
            overrides: vec![OverrideBlock {
                paths: vec!["**/*.md".to_string()],
                extra_words: Some(vec!["from_global".to_string()]),
                ..OverrideBlock::default_for_test()
            }],
            ..Default::default()
        };

        let project = ConfigSettings {
            words: vec!["project".to_string()],
            overrides: vec![OverrideBlock {
                paths: vec!["**/*.md".to_string()],
                extra_words: Some(vec!["from_project".to_string()]),
                ..OverrideBlock::default_for_test()
            }],
            ..Default::default()
        };

        global.merge(project);

        // Overrides should be: global first, then project
        assert_eq!(global.overrides.len(), 2);
        assert_eq!(
            global.overrides[0].extra_words,
            Some(vec!["from_global".to_string()])
        );
        assert_eq!(
            global.overrides[1].extra_words,
            Some(vec!["from_project".to_string()])
        );
    }

    #[test]
    fn test_serialization_with_overrides() {
        let config = ConfigSettings {
            words: vec!["base".to_string()],
            overrides: vec![OverrideBlock {
                paths: vec!["**/*.md".to_string()],
                extra_words: Some(vec!["markdown".to_string()]),
                ..OverrideBlock::default_for_test()
            }],
            ..Default::default()
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: ConfigSettings = toml::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_settings_query_methods() {
        let settings = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            words: vec!["codebook".to_string()],
            flag_words: vec!["todo".to_string()],
            min_word_length: Some(4),
            ..Default::default()
        };

        assert_eq!(settings.dictionary_ids(), vec!["en_us"]);
        assert!(settings.is_allowed_word("codebook"));
        assert!(settings.is_allowed_word("CODEBOOK")); // case insensitive
        assert!(!settings.is_allowed_word("unknown"));
        assert!(settings.should_flag_word("todo"));
        assert!(settings.should_flag_word("TODO")); // case insensitive
        assert!(!settings.should_flag_word("done"));
        assert_eq!(settings.min_word_length(), 4);
    }

    #[test]
    fn test_dictionary_ids_default() {
        let settings = ConfigSettings::default();
        assert_eq!(settings.dictionary_ids(), vec!["en_us"]);
    }

    impl OverrideBlock {
        /// Helper for tests: creates an OverrideBlock with all fields set to None/empty.
        fn default_for_test() -> Self {
            Self {
                paths: vec![],
                dictionaries: None,
                words: None,
                flag_words: None,
                ignore_patterns: None,
                extra_dictionaries: None,
                extra_words: None,
                extra_flag_words: None,
                extra_ignore_patterns: None,
            }
        }
    }
}
