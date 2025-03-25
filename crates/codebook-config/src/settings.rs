use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ConfigSettings {
    /// List of dictionaries to use for spell checking
    #[serde(default)]
    pub dictionaries: Vec<String>,

    /// Custom allowlist of words
    #[serde(default)]
    pub words: Vec<String>,

    /// Words that should always be flagged
    #[serde(default)]
    pub flag_words: Vec<String>,

    /// Glob patterns for paths to ignore
    #[serde(default)]
    pub ignore_paths: Vec<String>,

    /// Regex patterns for text to ignore
    #[serde(default)]
    pub ignore_patterns: Vec<String>,

    /// Whether to use global configuration
    #[serde(default = "default_use_global")]
    pub use_global: bool,
}

fn default_use_global() -> bool {
    true
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
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(ConfigSettings {
            dictionaries: to_lowercase_vec(helper.dictionaries),
            words: to_lowercase_vec(helper.words),
            flag_words: to_lowercase_vec(helper.flag_words),
            ignore_paths: helper.ignore_paths,
            ignore_patterns: helper.ignore_patterns,
            use_global: helper.use_global,
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
        self.ignore_paths.extend(other.ignore_paths);
        self.ignore_patterns.extend(other.ignore_patterns);

        // The use_global setting from the other config is ignored during merging
        // as this is a per-config setting

        // Sort and deduplicate each collection
        self.sort_and_dedup();
    }

    /// Sort and deduplicate all collections in the config
    pub fn sort_and_dedup(&mut self) {
        // Sort and deduplicate each Vec
        sort_and_dedup(&mut self.dictionaries);
        sort_and_dedup(&mut self.words);
        sort_and_dedup(&mut self.flag_words);
        sort_and_dedup(&mut self.ignore_paths);
        sort_and_dedup(&mut self.ignore_patterns);
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
    fn test_serialization() {
        let config = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            words: vec!["rust".to_string()],
            ..Default::default()
        };

        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("dictionaries = [\"en_us\"]"));
        assert!(serialized.contains("words = [\"rust\"]"));
        assert!(serialized.contains("use_global = true"));
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
        };

        let other = ConfigSettings {
            dictionaries: vec!["en_gb".to_string(), "en_us".to_string()],
            words: vec!["rust".to_string()],
            flag_words: vec!["fixme".to_string()],
            ignore_paths: vec!["target/".to_string()],
            ignore_patterns: vec!["^//.*$".to_string()],
            use_global: false,
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
}
