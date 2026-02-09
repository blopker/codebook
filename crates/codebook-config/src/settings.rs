use glob::Pattern;
use log::warn;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single `[[overrides]]` block in the config file.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct OverrideBlock {
    /// Required: glob patterns matched against file path relative to project root
    pub paths: Vec<String>,

    // --- Replace fields (replace the base list entirely) ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dictionaries: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag_words: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,

    // --- Append fields (append to the resolved list) ---
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_dictionaries: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_words: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_flag_words: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_ignore_patterns: Option<Vec<String>>,
}

impl<'de> Deserialize<'de> for OverrideBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn to_lowercase_vec(v: Vec<String>) -> Vec<String> {
            v.into_iter().map(|s| s.to_ascii_lowercase()).collect()
        }

        fn to_lowercase_opt(v: Option<Vec<String>>) -> Option<Vec<String>> {
            v.map(to_lowercase_vec)
        }

        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            paths: Vec<String>,
            #[serde(default)]
            dictionaries: Option<Vec<String>>,
            #[serde(default)]
            words: Option<Vec<String>>,
            #[serde(default)]
            flag_words: Option<Vec<String>>,
            #[serde(default)]
            ignore_patterns: Option<Vec<String>>,
            #[serde(default)]
            extra_dictionaries: Option<Vec<String>>,
            #[serde(default)]
            extra_words: Option<Vec<String>>,
            #[serde(default)]
            extra_flag_words: Option<Vec<String>>,
            #[serde(default)]
            extra_ignore_patterns: Option<Vec<String>>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(OverrideBlock {
            paths: helper.paths,
            // Lowercase word-related fields
            dictionaries: to_lowercase_opt(helper.dictionaries),
            words: to_lowercase_opt(helper.words),
            flag_words: to_lowercase_opt(helper.flag_words),
            extra_dictionaries: to_lowercase_opt(helper.extra_dictionaries),
            extra_words: to_lowercase_opt(helper.extra_words),
            extra_flag_words: to_lowercase_opt(helper.extra_flag_words),
            // Don't lowercase patterns or paths
            ignore_patterns: helper.ignore_patterns,
            extra_ignore_patterns: helper.extra_ignore_patterns,
        })
    }
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
        let path_str = relative_path.to_string_lossy();
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

    /// Scoped configuration overrides
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

fn is_default_min_word_length(value: &usize) -> bool {
    *value == default_min_word_length()
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            dictionaries: vec![],
            words: Vec::new(),
            flag_words: Vec::new(),
            ignore_paths: Vec::new(),
            ignore_patterns: Vec::new(),
            use_global: true,
            min_word_length: default_min_word_length(),
            overrides: Vec::new(),
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
            ignore_paths: Vec<String>,
            #[serde(default)]
            ignore_patterns: Vec<String>,
            #[serde(default = "default_use_global")]
            use_global: bool,
            #[serde(default = "default_min_word_length")]
            min_word_length: usize,
            #[serde(default)]
            overrides: Vec<OverrideBlock>,
        }

        let helper = Helper::deserialize(deserializer)?;

        // Filter out invalid override blocks
        let overrides: Vec<OverrideBlock> = helper
            .overrides
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
            .collect();

        Ok(ConfigSettings {
            dictionaries: to_lowercase_vec(helper.dictionaries),
            words: to_lowercase_vec(helper.words),
            flag_words: to_lowercase_vec(helper.flag_words),
            ignore_paths: helper.ignore_paths,
            ignore_patterns: helper.ignore_patterns,
            use_global: helper.use_global,
            min_word_length: helper.min_word_length,
            overrides,
        })
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
        self.ignore_paths.extend(other.ignore_paths);
        self.ignore_patterns.extend(other.ignore_patterns);

        // Append overrides (global first, then project — order matters)
        self.overrides.extend(other.overrides);

        // The use_global setting from the other config is ignored during merging
        // as this is a per-config setting

        // Override min_word_length if the other config has a non-default value
        if other.min_word_length != default_min_word_length() {
            self.min_word_length = other.min_word_length;
        }

        // Sort and deduplicate each collection (but NOT overrides)
        self.sort_and_dedup();
    }

    /// Sort and deduplicate all collections in the config (but not overrides).
    pub fn sort_and_dedup(&mut self) {
        // Sort and deduplicate each Vec
        sort_and_dedup(&mut self.dictionaries);
        sort_and_dedup(&mut self.words);
        sort_and_dedup(&mut self.flag_words);
        sort_and_dedup(&mut self.ignore_paths);
        sort_and_dedup(&mut self.ignore_patterns);
        // Note: overrides are NOT sorted — order matters for resolution
    }

    /// Apply a single override block to this settings (mutates in place).
    /// Replace fields are applied first, then append fields.
    pub fn apply_override(&mut self, ovr: &OverrideBlock) {
        // Replace fields: fully replace the list
        if let Some(ref v) = ovr.dictionaries {
            self.dictionaries = v.clone();
        }
        if let Some(ref v) = ovr.words {
            self.words = v.clone();
        }
        if let Some(ref v) = ovr.flag_words {
            self.flag_words = v.clone();
        }
        if let Some(ref v) = ovr.ignore_patterns {
            self.ignore_patterns = v.clone();
        }

        // Append fields: extend the current list
        if let Some(ref v) = ovr.extra_dictionaries {
            self.dictionaries.extend(v.clone());
        }
        if let Some(ref v) = ovr.extra_words {
            self.words.extend(v.clone());
        }
        if let Some(ref v) = ovr.extra_flag_words {
            self.flag_words.extend(v.clone());
        }
        if let Some(ref v) = ovr.extra_ignore_patterns {
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

    /// Get dictionary IDs, providing a default when none are configured.
    pub fn dictionary_ids(&self) -> Vec<String> {
        if self.dictionaries.is_empty() {
            vec!["en_us".to_string()]
        } else {
            self.dictionaries.clone()
        }
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

    /// Get the minimum word length to check.
    pub fn get_min_word_length(&self) -> usize {
        self.min_word_length
    }
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
        assert_eq!(config.ignore_paths, Vec::<String>::new());
        assert_eq!(config.ignore_patterns, Vec::<String>::new());
        assert!(config.use_global);
        assert_eq!(config.min_word_length, 3);
        assert!(config.overrides.is_empty());
    }

    #[test]
    fn test_deserialization() {
        let toml_str = r#"
        dictionaries = ["EN_US", "en_GB"]
        words = ["CodeBook", "Rust"]
        flag_words = ["TODO", "FIXME"]
        ignore_paths = ["**/*.md", "target/"]
        ignore_patterns = ["^```.*$", "^//.*$"]
        use_global = false
        "#;

        let config: ConfigSettings = toml::from_str(toml_str).unwrap();

        assert_eq!(config.dictionaries, vec!["en_us", "en_gb"]);
        assert_eq!(config.words, vec!["codebook", "rust"]);
        assert_eq!(config.flag_words, vec!["todo", "fixme"]);
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
    fn test_merge() {
        let mut base = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            words: vec!["codebook".to_string()],
            flag_words: vec!["todo".to_string()],
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

    #[test]
    fn test_apply_override_replace() {
        let mut settings = ConfigSettings {
            words: vec!["alpha".to_string(), "beta".to_string()],
            ..Default::default()
        };

        let ovr = OverrideBlock {
            paths: vec!["**/*.md".to_string()],
            words: Some(vec!["gamma".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        settings.apply_override(&ovr);
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

        let ovr = OverrideBlock {
            paths: vec!["**/*.md".to_string()],
            extra_flag_words: Some(vec!["hack".to_string()]),
            ..OverrideBlock::default_for_test()
        };

        settings.apply_override(&ovr);
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
            min_word_length: 4,
            ..Default::default()
        };

        assert_eq!(settings.dictionary_ids(), vec!["en_us"]);
        assert!(settings.is_allowed_word("codebook"));
        assert!(settings.is_allowed_word("CODEBOOK")); // case insensitive
        assert!(!settings.is_allowed_word("unknown"));
        assert!(settings.should_flag_word("todo"));
        assert!(settings.should_flag_word("TODO")); // case insensitive
        assert!(!settings.should_flag_word("done"));
        assert_eq!(settings.get_min_word_length(), 4);
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
