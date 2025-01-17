use anyhow::{Context, Result};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CodebookConfig {
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
}

impl CodebookConfig {
    /// Load configuration by searching up from the current directory
    pub fn load() -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        Self::find_and_load_config(&current_dir)
    }

    /// Load configuration starting from a specific directory
    pub fn load_from_dir<P: AsRef<Path>>(start_dir: P) -> Result<Self> {
        Self::find_and_load_config(start_dir.as_ref())
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
        Ok(Self::default())
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;

        Ok(config)
    }

    /// Check if a path should be ignored based on the ignore_paths patterns
    pub fn should_ignore_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        self.ignore_paths.iter().any(|pattern| {
            Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
        })
    }

    /// Check if a word is in the custom allowlist
    pub fn is_allowed_word(&self, word: &str) -> bool {
        self.words.iter().any(|w| w == word)
    }

    /// Check if a word should be flagged
    pub fn should_flag_word(&self, word: &str) -> bool {
        self.flag_words.iter().any(|w| w == word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_config_recursive_search() -> Result<()> {
        // Create a temporary directory structure
        let temp_dir = TempDir::new()?;
        let sub_dir = temp_dir.path().join("sub");
        let sub_sub_dir = sub_dir.join("subsub");
        fs::create_dir_all(&sub_sub_dir)?;

        // Create a config file in the root temp directory
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

        // Test finding config from different directories
        let config = CodebookConfig::load_from_dir(&sub_sub_dir)?;
        assert!(config.words.contains(&"testword".to_string()));

        Ok(())
    }
}
