use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct CustomDictionariesDefinitions {
    /// The name of the custom dictionary
    name: String,

    /// Relative path to the custom dictionary
    path: String,

    /// Allow adding words to this dictionary
    allow_add_words: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ConfigSettings {
    /// List of dictionaries to use for spell checking
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dictionaries: Vec<String>,

    /// List of custom dictionaries to use for spell checking
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_dictionaries_definitions: Vec<CustomDictionariesDefinitions>,

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
            custom_dictionaries_definitions: vec![],
            words: Vec::new(),
            flag_words: Vec::new(),
            ignore_paths: Vec::new(),
            ignore_patterns: Vec::new(),
            use_global: true,
            min_word_length: default_min_word_length(),
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
            custom_dictionaries_definitions: Vec<CustomDictionariesDefinitions>,
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
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(ConfigSettings {
            dictionaries: to_lowercase_vec(helper.dictionaries),
            custom_dictionaries_definitions: helper.custom_dictionaries_definitions,
            words: to_lowercase_vec(helper.words),
            flag_words: to_lowercase_vec(helper.flag_words),
            ignore_paths: helper.ignore_paths,
            ignore_patterns: helper.ignore_patterns,
            use_global: helper.use_global,
            min_word_length: helper.min_word_length,
        })
    }
}

impl ConfigSettings {
    /// Merge another config settings into this one, sorting and deduplicating all collections, prioritizing self when possible
    pub fn merge(&mut self, other: ConfigSettings) {
        // Add items from the other config
        self.dictionaries.extend(other.dictionaries);
        self.custom_dictionaries_definitions
            .extend(other.custom_dictionaries_definitions);
        self.words.extend(other.words);
        self.flag_words.extend(other.flag_words);
        self.ignore_paths.extend(other.ignore_paths);
        self.ignore_patterns.extend(other.ignore_patterns);

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
        sort_and_dedup_by(&mut self.custom_dictionaries_definitions, |d1, d2| {
            d1.name.cmp(&d2.name)
        });
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

pub fn sort_and_dedup_by<T, F>(vec: &mut Vec<T>, f: F)
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    vec.sort_by(&f);
    vec.dedup_by(|d1, d2| f(d1, d2) == std::cmp::Ordering::Equal);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_fake_custom_dict(name: &str) -> CustomDictionariesDefinitions {
        CustomDictionariesDefinitions {
            name: name.into(),
            path: name.into(),
            allow_add_words: false,
        }
    }

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
        let mut duplicate_custom_dict = build_fake_custom_dict("duplicate");

        let mut base = ConfigSettings {
            dictionaries: vec!["en_us".to_string()],
            custom_dictionaries_definitions: vec![
                build_fake_custom_dict("base_unique"),
                duplicate_custom_dict.clone(),
            ],
            words: vec!["codebook".to_string()],
            flag_words: vec!["todo".to_string()],
            ignore_paths: vec!["**/*.md".to_string()],
            ignore_patterns: vec!["^```.*$".to_string()],
            use_global: true,
            min_word_length: 3,
        };

        // flip allow_add_words to true, to create a disparity between the dictionaries
        duplicate_custom_dict.allow_add_words = !duplicate_custom_dict.allow_add_words;

        let other = ConfigSettings {
            dictionaries: vec!["en_gb".to_string(), "en_us".to_string()],
            custom_dictionaries_definitions: vec![
                duplicate_custom_dict.clone(),
                build_fake_custom_dict("other_unique"),
            ],
            words: vec!["rust".to_string()],
            flag_words: vec!["fixme".to_string()],
            ignore_paths: vec!["target/".to_string()],
            ignore_patterns: vec!["^//.*$".to_string()],
            use_global: false,
            min_word_length: 2,
        };

        base.merge(other);

        // After merging and deduplicating, we should have combined items
        assert_eq!(base.dictionaries, vec!["en_gb", "en_us"]);
        assert_eq!(
            base.custom_dictionaries_definitions
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<String>>(),
            vec!["base_unique", "duplicate", "other_unique"]
        );
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

        // Assert that base custom_dictionaries_definitions took priority
        assert_ne!(
            base.custom_dictionaries_definitions.iter().find(|d| d.name == "duplicate").expect("custom_dictionaries_definitions duplicate must be present if set in ether of the merged dictionaries").allow_add_words 
            ,duplicate_custom_dict.allow_add_words
        );
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
            custom_dictionaries_definitions: vec![
                build_fake_custom_dict("custom_1"),
                build_fake_custom_dict("custom_2"),
                build_fake_custom_dict("custom_1"),
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
        };

        config.sort_and_dedup();

        assert_eq!(config.dictionaries, vec!["en_gb", "en_us"]);
        assert_eq!(
            config
                .custom_dictionaries_definitions
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<String>>(),
            vec!["custom_1", "custom_2"]
        );
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
