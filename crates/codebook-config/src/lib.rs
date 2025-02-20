use anyhow::{Context, Result};
use glob::Pattern;
use log::debug;
use log::info;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

static CACHE_DIR: &str = "codebook";

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
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            dictionaries: vec![],
            words: Vec::new(),
            flag_words: Vec::new(),
            ignore_paths: Vec::new(),
            ignore_patterns: Vec::new(),
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
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(ConfigSettings {
            dictionaries: to_lowercase_vec(helper.dictionaries),
            words: to_lowercase_vec(helper.words),
            flag_words: to_lowercase_vec(helper.flag_words),
            ignore_paths: helper.ignore_paths,
            ignore_patterns: helper.ignore_patterns,
        })
    }
}

#[derive(Debug)]
pub struct CodebookConfig {
    settings: RwLock<ConfigSettings>,
    regex_set: RwLock<Option<RegexSet>>,
    pub config_path: Option<PathBuf>,
    pub cache_dir: PathBuf,
}

impl Default for CodebookConfig {
    fn default() -> Self {
        Self {
            settings: RwLock::new(ConfigSettings::default()),
            regex_set: RwLock::new(None),
            config_path: None,
            cache_dir: env::temp_dir().join(CACHE_DIR),
        }
    }
}

impl CodebookConfig {
    /// Load configuration by searching up from the current directory
    pub fn load() -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        Self::find_and_load_config(&current_dir)
    }

    pub fn new_no_file() -> Self {
        Self::default()
    }

    pub fn clean_cache(&self) {
        let dir_path = self.cache_dir.clone();
        // Check if the path exists and is a directory
        if !dir_path.is_dir() {
            return;
        }

        // Read directory entries
        for entry in fs::read_dir(dir_path).unwrap() {
            let path = entry.unwrap().path();

            if path.is_dir() {
                // If it's a directory, recursively remove it
                fs::remove_dir_all(path).unwrap();
            } else {
                // If it's a file, remove it
                fs::remove_file(path).unwrap();
            }
        }
    }

    pub fn get_dictionary_ids(&self) -> Vec<String> {
        let ids = self.settings.read().unwrap().dictionaries.clone();
        if ids.is_empty() {
            return vec!["en_us".to_string()];
        }
        ids
    }

    pub fn reload(&self) -> Result<bool> {
        let config_path = match self.config_path.as_ref() {
            Some(c) => c,
            None => {
                debug!("config_path was never set, can't reload config.");
                return Ok(false);
            }
        };

        // get file contents or reset config to default, with the config_path set
        let new_settings = match fs::read_to_string(config_path) {
            Ok(content) => toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_path.display())),
            Err(_) => {
                info!("Failed to read config file, resetting to default config.");
                let new_settings = ConfigSettings::default();
                Ok(new_settings)
            }
        }?;
        let mut settings = self.settings.write().unwrap();
        if new_settings != *settings {
            info!("Reloading config from file: {}", config_path.display());
            *settings = new_settings;
            *self.regex_set.write().unwrap() = None;
            return Ok(true);
        }
        Ok(false)
    }

    /// Load configuration starting from a specific directory
    pub fn load_from_dir<P: AsRef<Path>>(start_dir: P) -> Result<Self> {
        Self::find_and_load_config(start_dir.as_ref())
    }

    /// Add a word to the allowlist and save the configuration
    pub fn add_word(&self, word: &str) -> Result<bool> {
        {
            let word = word.to_ascii_lowercase();
            let settings = &mut self.settings.write().unwrap();
            // Check if word already exists
            if settings.words.contains(&word.to_string()) {
                return Ok(false);
            }

            // Add the word
            settings.words.push(word.to_string());
            // Sort for consistency
            settings.words.sort();
        }
        Ok(true)
    }

    /// Save the configuration to its file
    pub fn save(&self) -> Result<()> {
        let config_path = match self.config_path.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };

        let content = self.as_toml()?;

        fs::write(config_path, content).with_context(|| {
            format!("Failed to write config to file: {}", config_path.display())
        })?;

        Ok(())
    }

    pub fn as_toml(&self) -> Result<String> {
        toml::to_string_pretty(&*self.settings.read().unwrap())
            .context("Failed to serialize config")
    }

    /// Create a new configuration file if one doesn't exist
    pub fn create_if_not_exists(directory: Option<&PathBuf>) -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let default_name = "codebook.toml";
        let config_path = match directory {
            Some(d) => d.join(default_name),
            None => current_dir.join(default_name),
        };

        if config_path.exists() {
            return Self::load_from_file(&config_path);
        }

        let config = Self {
            config_path: Some(config_path.clone()),
            ..Default::default()
        };

        // Save the new config
        let content = toml::to_string_pretty(&*config.settings.read().unwrap())
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to create config file: {}", config_path.display()))?;

        Ok(config)
    }

    /// Recursively search for and load config from the given directory and its parents
    fn find_and_load_config(start_dir: &Path) -> Result<Self> {
        let config_files = ["codebook.toml", ".codebook.toml"];

        // Start from the given directory and walk up to root
        let mut current_dir = Some(start_dir.to_path_buf());

        while let Some(dir) = current_dir {
            // Try each possible config filename in the current directory
            for config_name in &config_files {
                let config_path = dir.join(config_name);
                if config_path.is_file() {
                    return Self::load_from_file(&config_path);
                }
            }

            // Move to parent directory
            current_dir = dir.parent().map(PathBuf::from);
        }

        // If no config file is found, return default config
        let mut config = Self::default();
        config.config_path = Some(start_dir.join(config_files[0]));
        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let settings: ConfigSettings = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        let settings_arc = RwLock::new(settings);
        // Store the config file path
        let config = Self {
            settings: settings_arc,
            config_path: Some(path.to_path_buf()),
            ..Default::default()
        };

        Ok(config)
    }

    /// Check if a path should be ignored based on the ignore_paths patterns
    pub fn should_ignore_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        self.settings
            .read()
            .unwrap()
            .ignore_paths
            .iter()
            .any(|pattern| {
                Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            })
    }

    /// Check if a word is in the custom allowlist
    pub fn is_allowed_word(&self, word: &str) -> bool {
        if self.matches_ignore_pattern(word) {
            return true;
        }
        let word = word.to_ascii_lowercase();
        self.settings
            .read()
            .unwrap()
            .words
            .iter()
            .any(|w| w == &word)
    }

    /// Check if text matches any of the ignore patterns
    fn matches_ignore_pattern(&self, word: &str) -> bool {
        let patterns = &self.settings.read().unwrap().ignore_patterns;
        if patterns.is_empty() {
            return false;
        }

        // Lazily initialize the RegexSet
        let mut regex_set = self.regex_set.write().unwrap();
        if regex_set.is_none() {
            *regex_set = Some(RegexSet::new(patterns).unwrap());
        }

        // Check if text matches any pattern
        if let Some(set) = &*regex_set {
            return set.is_match(word);
        }
        false
    }

    /// Check if a word should be flagged
    pub fn should_flag_word(&self, word: &str) -> bool {
        let word = word.to_ascii_lowercase();
        self.settings
            .read()
            .unwrap()
            .flag_words
            .iter()
            .any(|w| w == &word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_add_word() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            config_path: Some(config_path.clone()),
            ..Default::default()
        };
        config.save()?;
        // Add a word
        config.add_word("testword")?;
        config.save()?;
        // Reload config and verify
        let loaded_config = CodebookConfig::load_from_file(&config_path)?;
        assert!(loaded_config.is_allowed_word("testword"));

        Ok(())
    }

    #[test]
    fn test_ignore_patterns() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("codebook.toml");
        let mut file = File::create(&config_path)?;
        let a = r#"
        ignore_patterns = [
            "^[ATCG]+$",
            "\\d{3}-\\d{2}-\\d{4}"  # Social Security Number format
        ]
        "#;
        file.write_all(a.as_bytes())?;

        let config = CodebookConfig::load_from_file(&config_path)?;
        assert!(config.matches_ignore_pattern("GTAC"));
        assert!(config.matches_ignore_pattern("AATTCCGG"));
        assert!(config.matches_ignore_pattern("123-45-6789"));
        assert!(!config.matches_ignore_pattern("Hello"));
        assert!(!config.matches_ignore_pattern("GTACZ")); // Invalid DNA sequence

        Ok(())
    }
    #[test]
    fn test_reload_ignore_patterns() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config with DNA pattern
        let mut file = File::create(&config_path)?;
        write!(
            file,
            r#"
            ignore_patterns = [
                "^[ATCG]+$"
            ]
            "#
        )?;

        let config = CodebookConfig::load_from_file(&config_path)?;
        assert!(config.matches_ignore_pattern("GTAC"));
        assert!(!config.matches_ignore_pattern("123-45-6789"));

        // Update config with new pattern
        let mut file = File::create(&config_path)?;
        let a = r#"
        ignore_patterns = [
            "^[ATCG]+$",
            "\\d{3}-\\d{2}-\\d{4}"
        ]
        "#;
        file.write_all(a.as_bytes())?;

        // Reload and verify both patterns work
        config.reload()?;
        assert!(config.matches_ignore_pattern("GTAC"));
        assert!(config.matches_ignore_pattern("123-45-6789"));

        // Update config to remove all patterns
        let mut file = File::create(&config_path)?;
        write!(
            file,
            r#"
            ignore_patterns = []
            "#
        )?;

        // Reload and verify no patterns match
        config.reload()?;
        assert!(!config.matches_ignore_pattern("GTAC"));
        assert!(!config.matches_ignore_pattern("123-45-6789"));

        Ok(())
    }
    #[test]
    fn test_config_recursive_search() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let sub_dir = temp_dir.path().join("sub");
        let sub_sub_dir = sub_dir.join("subsub");
        fs::create_dir_all(&sub_sub_dir)?;

        let config_path = temp_dir.path().join("codebook.toml");
        let mut file = File::create(&config_path)?;
        write!(
            file,
            r#"
            dictionaries = ["en_US"]
            words = ["testword"]
            flag_words = ["todo"]
            ignore_paths = ["target/**/*"]
        "#
        )?;

        let config = CodebookConfig::load_from_dir(&sub_sub_dir)?;
        assert!(config
            .settings
            .read()
            .unwrap()
            .words
            .contains(&"testword".to_string()));
        // Check that the config file path is stored
        assert_eq!(config.config_path, Some(config_path));
        Ok(())
    }

    #[test]
    fn test_create_if_not_exists() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_dir_path = temp_dir.path().to_path_buf();
        let config_path = config_dir_path.join("codebook.toml");

        // Create a new config file
        let config = CodebookConfig::create_if_not_exists(Some(&config_dir_path))?;
        assert_eq!(config.config_path, Some(config_path.clone()));

        // Check that the file was created
        assert!(config_path.exists());

        // Check that the file can be loaded
        let loaded_config = CodebookConfig::load_from_file(&config_path)?;
        assert_eq!(
            config.settings.read().unwrap().clone(),
            loaded_config.settings.read().unwrap().clone()
        );

        Ok(())
    }

    #[test]
    fn test_should_ignore_path() -> Result<()> {
        let config = CodebookConfig::default();
        config
            .settings
            .write()
            .unwrap()
            .ignore_paths
            .push("target/**/*".to_string());
        assert!(config.should_ignore_path("target/debug/build"));
        assert!(!config.should_ignore_path("src/main.rs"));

        Ok(())
    }
    #[test]
    fn test_reload() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            config_path: Some(config_path.clone()),
            ..Default::default()
        };
        config.save()?;

        // Add a word to the toml file
        let mut file = File::create(&config_path)?;
        write!(
            file,
            r#"
            words = ["testword"]
            "#
        )?;

        // Reload config and verify
        config.reload()?;
        assert!(config.is_allowed_word("testword"));

        Ok(())
    }

    #[test]
    fn test_reload_when_deleted() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            config_path: Some(config_path.clone()),
            ..Default::default()
        };
        config.save()?;

        // Add a word to the toml file
        let mut file = File::create(&config_path)?;
        write!(
            file,
            r#"
            words = ["testword"]
            "#
        )?;

        // Reload config and verify
        config.reload()?;
        assert!(config.is_allowed_word("testword"));

        // Delete the config file
        fs::remove_file(&config_path)?;

        // Reload config and verify
        config.reload().unwrap();
        assert!(!config.is_allowed_word("testword"));

        Ok(())
    }

    #[test]
    fn test_add_word_case() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            config_path: Some(config_path.clone()),
            ..Default::default()
        };
        config.save()?;
        // Add a word with mixed case
        config.add_word("TestWord")?;
        config.save()?;

        // Reload config and verify with different cases
        let loaded_config = CodebookConfig::load_from_file(&config_path)?;
        assert!(loaded_config.is_allowed_word("testword"));
        assert!(loaded_config.is_allowed_word("TESTWORD"));
        assert!(loaded_config.is_allowed_word("TestWord"));

        Ok(())
    }
}
