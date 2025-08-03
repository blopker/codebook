mod settings;
use crate::settings::ConfigSettings;
use glob::Pattern;
use log::debug;
use log::info;
use regex::Regex;
use std::env;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;

static CACHE_DIR: &str = "codebook";
static GLOBAL_CONFIG_FILE: &str = "codebook.toml";
static USER_CONFIG_FILES: [&str; 2] = ["codebook.toml", ".codebook.toml"];

#[derive(Debug)]
struct ConfigFileState {
    last_modified: SystemTime,
    last_size: u64,
}

#[derive(Debug)]
pub struct CodebookConfig {
    /// Project-specific settings
    project_settings: RwLock<ConfigSettings>,
    /// Global settings (from user config directory)
    global_settings: RwLock<Option<ConfigSettings>>,
    /// Combined settings (global merged with project overrides)
    effective_settings: RwLock<ConfigSettings>,
    /// Compiled regex patterns for ignoring text
    regex_cache: RwLock<Option<Vec<Regex>>>,
    /// Path to the project-specific config file
    pub project_config_path: Option<PathBuf>,
    project_config_state: RwLock<Option<ConfigFileState>>,
    /// Path to the global config file
    pub global_config_path: Option<PathBuf>,
    global_config_state: RwLock<Option<ConfigFileState>>,
    /// Directory for caching
    pub cache_dir: PathBuf,
}

impl Default for CodebookConfig {
    fn default() -> Self {
        Self {
            project_settings: RwLock::new(ConfigSettings::default()),
            global_settings: RwLock::new(None),
            effective_settings: RwLock::new(ConfigSettings::default()),
            regex_cache: RwLock::new(None),
            project_config_path: None,
            project_config_state: RwLock::new(None),
            global_config_path: None,
            global_config_state: RwLock::new(None),
            cache_dir: env::temp_dir().join(CACHE_DIR),
        }
    }
}

impl CodebookConfig {
    /// Load configuration by searching for both global and project-specific configs
    pub fn load(current_dir: Option<&Path>) -> Result<Self, io::Error> {
        debug!("Initializing CodebookConfig");

        if let Some(current_dir) = current_dir {
            let current_dir = Path::new(current_dir);
            Self::load_configs(current_dir)
        } else {
            let current_dir = env::current_dir()?;
            Self::load_configs(&current_dir)
        }
    }

    /// Load both global and project configuration
    fn load_configs(start_dir: &Path) -> Result<Self, io::Error> {
        // Start with the default configuration
        let mut config = Self::default();

        // Try to load global config first
        if let Some(global_path) = Self::find_global_config_path() {
            if global_path.exists() {
                match Self::load_settings_from_file(&global_path) {
                    Ok(global_settings) => {
                        debug!("Loaded global config from {}", global_path.display());
                        config.global_config_path = Some(global_path.clone());
                        *config.global_settings.write().unwrap() = Some(global_settings.clone());
                        *config.effective_settings.write().unwrap() = global_settings;

                        // Initialize file state
                        if let Ok(metadata) = fs::metadata(&global_path) {
                            if let Ok(modified) = metadata.modified() {
                                *config.global_config_state.write().unwrap() =
                                    Some(ConfigFileState {
                                        last_modified: modified,
                                        last_size: metadata.len(),
                                    });
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to load global config: {e}");
                    }
                }
            } else {
                info!("No global config found, using default");
                config.global_config_path = Some(global_path);
            }
        }

        // Then try to find and load project config
        if let Some((project_path, project_settings)) = Self::find_project_config(start_dir)? {
            debug!("Loaded project config from {}", project_path.display());
            config.project_config_path = Some(project_path.clone());
            *config.project_settings.write().unwrap() = project_settings.clone();

            // Initialize file state
            if let Ok(metadata) = fs::metadata(&project_path) {
                if let Ok(modified) = metadata.modified() {
                    *config.project_config_state.write().unwrap() = Some(ConfigFileState {
                        last_modified: modified,
                        last_size: metadata.len(),
                    });
                }
            }

            // If use_global is true, merge global with project (project takes precedence)
            // Otherwise, just use project settings
            if project_settings.use_global {
                if let Some(global_settings) = config.global_settings.read().unwrap().as_ref() {
                    let mut effective = global_settings.clone();
                    effective.merge(project_settings);
                    *config.effective_settings.write().unwrap() = effective;
                } else {
                    *config.effective_settings.write().unwrap() = project_settings;
                }
            } else {
                *config.effective_settings.write().unwrap() = project_settings;
            }
        } else {
            info!("No project config found, using default");
            // Set path to start_dir if no config is found
            config.project_config_path = Some(start_dir.join(USER_CONFIG_FILES[0]));
        }

        Ok(config)
    }
    /// Find the platform-specific global config directory and file path
    fn find_global_config_path() -> Option<PathBuf> {
        // On Linux/macOS XDG_CONFIG_HOME, fallback to ~/.config
        if cfg!(unix) {
            // First try XDG_CONFIG_HOME environment variable (Linux/macOS)
            if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
                let path = PathBuf::from(xdg_config_home)
                    .join("codebook")
                    .join(GLOBAL_CONFIG_FILE);
                return Some(path);
            }
            if let Some(home) = dirs::home_dir() {
                let path = home
                    .join(".config")
                    .join("codebook")
                    .join(GLOBAL_CONFIG_FILE);
                return Some(path);
            }
        }

        // On Windows, use dirs::config_dir() (typically %APPDATA%)
        if cfg!(windows) {
            if let Some(config_dir) = dirs::config_dir() {
                return Some(config_dir.join("codebook").join(GLOBAL_CONFIG_FILE));
            }
        }

        None
    }

    /// Find project configuration by searching up from the current directory
    fn find_project_config(
        start_dir: &Path,
    ) -> Result<Option<(PathBuf, ConfigSettings)>, io::Error> {
        let config_files = USER_CONFIG_FILES;

        // Start from the given directory and walk up to root
        let mut current_dir = Some(start_dir.to_path_buf());

        while let Some(dir) = current_dir {
            // Try each possible config filename in the current directory
            for config_name in &config_files {
                let config_path = dir.join(config_name);
                if config_path.is_file() {
                    match Self::load_settings_from_file(&config_path) {
                        Ok(settings) => return Ok(Some((config_path, settings))),
                        Err(e) => return Err(e),
                    }
                }
            }

            // Move to parent directory
            current_dir = dir.parent().map(PathBuf::from);
        }

        Ok(None)
    }

    /// Load settings from a file
    fn load_settings_from_file<P: AsRef<Path>>(path: P) -> Result<ConfigSettings, io::Error> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;

        match toml::from_str(&content) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                let err = io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to parse config file {}: {e}", path.display()),
                );
                Err(err)
            }
        }
    }

    /// Reload both global and project configurations, only reading files if they've changed
    pub fn reload(&self) -> Result<bool, io::Error> {
        let mut changed = false;

        // Reload global config if it exists
        if let Some(global_path) = &self.global_config_path {
            if self.has_file_changed(global_path, &self.global_config_state)? {
                match fs::read_to_string(global_path) {
                    Ok(content) => {
                        if let Ok(new_settings) = toml::from_str::<ConfigSettings>(&content) {
                            let mut global_settings = self.global_settings.write().unwrap();
                            if Some(&new_settings) != global_settings.as_ref() {
                                *global_settings = Some(new_settings);
                                changed = true;
                            }

                            // Update state cache
                            self.update_file_state(global_path, &self.global_config_state)?;
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read global config: {e}");
                        // File might be deleted, clear the state
                        *self.global_config_state.write().unwrap() = None;
                    }
                }
            }
        }

        // Reload project config if it exists
        if let Some(project_path) = &self.project_config_path {
            if self.has_file_changed(project_path, &self.project_config_state)? {
                match fs::read_to_string(project_path) {
                    Ok(content) => {
                        if let Ok(new_settings) = toml::from_str::<ConfigSettings>(&content) {
                            let mut project_settings = self.project_settings.write().unwrap();
                            if new_settings != *project_settings {
                                *project_settings = new_settings;
                                changed = true;
                            }

                            // Update state cache
                            self.update_file_state(project_path, &self.project_config_state)?;
                        }
                    }
                    Err(e) => {
                        debug!("Failed to read project config: {e}");
                        // Reset project settings to default if file can't be read
                        let mut project_settings = self.project_settings.write().unwrap();
                        if *project_settings != ConfigSettings::default() {
                            *project_settings = ConfigSettings::default();
                            changed = true;
                        }

                        // Clear the state
                        *self.project_config_state.write().unwrap() = None;
                    }
                }
            }
        }

        // Recalculate effective settings if anything changed
        if changed {
            self.recalculate_effective_settings();
        }

        Ok(changed)
    }

    /// Update file state cache after successfully reading a file
    fn update_file_state(
        &self,
        path: &Path,
        state: &RwLock<Option<ConfigFileState>>,
    ) -> Result<(), io::Error> {
        if let Ok(metadata) = fs::metadata(path) {
            let modified = metadata.modified()?;
            let size = metadata.len();

            *state.write().unwrap() = Some(ConfigFileState {
                last_modified: modified,
                last_size: size,
            });
        }
        Ok(())
    }

    /// Check if a file has changed since we last read it
    fn has_file_changed(
        &self,
        path: &Path,
        state: &RwLock<Option<ConfigFileState>>,
    ) -> Result<bool, io::Error> {
        match fs::metadata(path) {
            Ok(metadata) => {
                let current_modified = metadata.modified()?;
                let current_size = metadata.len();

                let state_guard = state.read().unwrap();
                if let Some(cached_state) = &*state_guard {
                    // Check if file has been modified or size has changed
                    if current_modified > cached_state.last_modified
                        || current_size != cached_state.last_size
                    {
                        return Ok(true);
                    }
                    return Ok(false);
                }
                // No cached state means we need to read the file
                Ok(true)
            }
            Err(_) => {
                // File doesn't exist or can't be accessed
                // If we previously had a state, the file has changed (likely deleted)
                Ok(state.read().unwrap().is_some())
            }
        }
    }

    /// Recalculate the effective settings based on global and project settings
    fn recalculate_effective_settings(&self) {
        let mut effective = self.effective_settings.write().unwrap();
        let project = self.project_settings.read().unwrap();

        if project.use_global {
            if let Some(global) = self.global_settings.read().unwrap().as_ref() {
                let mut new_effective = global.clone();
                new_effective.merge(project.clone());
                *effective = new_effective;
            } else {
                *effective = project.clone();
            }
        } else {
            *effective = project.clone();
        }

        // Invalidate regex cache
        *self.regex_cache.write().unwrap() = None;
    }

    /// Add a word to the project configs allowlist
    pub fn add_word(&self, word: &str) -> Result<bool, io::Error> {
        {
            let word = word.to_ascii_lowercase();
            let mut project_settings = self.project_settings.write().unwrap();

            // Check if word already exists
            if project_settings.words.contains(&word) {
                return Ok(false);
            }

            // Add the word
            project_settings.words.push(word);
            // Sort/dedup for consistency
            project_settings.words.sort();
            project_settings.words.dedup();
        }
        self.recalculate_effective_settings();

        Ok(true)
    }
    /// Add a word to the global configs allowlist
    pub fn add_word_global(&self, word: &str) -> Result<bool, io::Error> {
        {
            let word = word.to_ascii_lowercase();
            let mut global_settings = self.global_settings.write().unwrap();

            let global_config = match global_settings.as_mut() {
                Some(config) => config,
                None => {
                    *global_settings = Some(ConfigSettings::default());
                    global_settings.as_mut().unwrap()
                }
            };

            // Check if word already exists
            if global_config.words.contains(&word) {
                return Ok(false);
            }

            // Add the word
            global_config.words.push(word);
            // Sort/dedup for consistency
            global_config.words.sort();
            global_config.words.dedup();
        }
        self.recalculate_effective_settings();

        Ok(true)
    }

    /// Add a file to the ignore list
    pub fn add_ignore(&self, file: &str) -> Result<bool, io::Error> {
        {
            let mut project_settings = self.project_settings.write().unwrap();
            let file = file.to_string();
            // Check if file already exists
            if project_settings.ignore_paths.contains(&file) {
                return Ok(false);
            }

            // Add the file
            project_settings.ignore_paths.push(file);
            // Sort/dedup for consistency
            project_settings.ignore_paths.sort();
            project_settings.ignore_paths.dedup();
        }
        self.recalculate_effective_settings();

        Ok(true)
    }

    /// Save the project configuration to its file
    pub fn save(&self) -> Result<(), io::Error> {
        let project_config_path = match self.project_config_path.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };

        let content = toml::to_string_pretty(&*self.project_settings.read().unwrap())
            .map_err(io::Error::other)?;
        info!(
            "Saving project configuration to {}",
            project_config_path.display()
        );
        fs::write(project_config_path, content)
    }

    /// Save the global configuration to its file
    pub fn save_global(&self) -> Result<(), io::Error> {
        let global_config_path = match self.global_config_path.as_ref() {
            Some(c) => c,
            None => return Ok(()),
        };

        let content = toml::to_string_pretty(&*self.global_settings.read().unwrap())
            .map_err(io::Error::other)?;
        info!(
            "Saving global configuration to {}",
            global_config_path.display()
        );
        // Create parent directories if they don't exist
        if let Some(parent) = global_config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(global_config_path, content)
    }

    /// Get dictionary IDs from effective configuration
    pub fn get_dictionary_ids(&self) -> Vec<String> {
        let ids = self.effective_settings.read().unwrap().dictionaries.clone();
        if ids.is_empty() {
            return vec!["en_us".to_string()];
        }
        ids
    }

    /// Check if a path should be ignored based on the effective configuration
    pub fn should_ignore_path<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        self.effective_settings
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

    /// Check if a word is in the effective allowlist
    pub fn is_allowed_word(&self, word: &str) -> bool {
        let word = word.to_ascii_lowercase();
        self.effective_settings
            .read()
            .unwrap()
            .words
            .iter()
            .any(|w| w == &word)
    }

    /// Check if a word should be flagged according to effective configuration
    pub fn should_flag_word(&self, word: &str) -> bool {
        let word = word.to_ascii_lowercase();
        self.effective_settings
            .read()
            .unwrap()
            .flag_words
            .iter()
            .any(|w| w == &word)
    }

    /// Get the list of user-defined ignore patterns
    pub fn get_ignore_patterns(&self) -> Option<Vec<Regex>> {
        let str_patterns = self
            .effective_settings
            .read()
            .unwrap()
            .ignore_patterns
            .clone();

        // Lazily initialize the Regex cache
        let mut regex_cache = self.regex_cache.write().unwrap();
        if regex_cache.is_none() {
            let regex_set = str_patterns
                .into_iter()
                .filter_map(|pattern| match Regex::new(&pattern) {
                    Ok(regex) => Some(regex),
                    Err(e) => {
                        log::error!("Ignoring invalid regex pattern '{pattern}': {e}");
                        None
                    }
                })
                .collect::<Vec<_>>();
            *regex_cache = Some(regex_set);
        }

        regex_cache.clone()
    }

    /// Clean the cache directory
    pub fn clean_cache(&self) {
        let dir_path = self.cache_dir.clone();
        // Check if the path exists and is a directory
        if !dir_path.is_dir() {
            return;
        }

        // Safety check: Ensure CACHE_DIR is in the path
        let path_str = dir_path.to_string_lossy();
        if !path_str.contains(CACHE_DIR) {
            log::error!(
                "Cache directory path '{path_str}' doesn't contain '{CACHE_DIR}', refusing to clean"
            );
            return;
        }

        // Read directory entries
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    // If it's a directory, recursively remove it
                    let _ = fs::remove_dir_all(path);
                } else {
                    // If it's a file, remove it
                    let _ = fs::remove_file(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[derive(Debug, Clone, Copy)]
    pub enum ConfigType {
        Project,
        Global,
    }

    // Helper function for tests
    fn load_from_file<P: AsRef<Path>>(
        config_type: ConfigType,
        path: P,
    ) -> Result<CodebookConfig, io::Error> {
        let mut config = CodebookConfig::default();

        match config_type {
            ConfigType::Project => {
                if let Ok(settings) = CodebookConfig::load_settings_from_file(&path) {
                    config.project_config_path = Some(path.as_ref().to_path_buf());
                    *config.project_settings.write().unwrap() = settings.clone();
                    *config.effective_settings.write().unwrap() = settings;
                }
            }
            ConfigType::Global => {
                if let Ok(settings) = CodebookConfig::load_settings_from_file(&path) {
                    config.global_config_path = Some(path.as_ref().to_path_buf());
                    *config.global_settings.write().unwrap() = Some(settings.clone());
                    *config.effective_settings.write().unwrap() = settings;
                }
            }
        }

        Ok(config)
    }

    #[test]
    fn test_save_global_creates_directories() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let global_dir = temp_dir.path().join("deep").join("nested").join("dir");
        let config_path = global_dir.join("codebook.toml");

        // Create config with a path that doesn't exist yet
        let config = CodebookConfig {
            global_config_path: Some(config_path.clone()),
            global_settings: RwLock::new(Some(ConfigSettings::default())),
            ..Default::default()
        };

        // Directory doesn't exist yet
        assert!(!global_dir.exists());

        // Save should create directories
        config.save_global()?;

        // Now directory and file should exist
        assert!(global_dir.exists());
        assert!(config_path.exists());

        Ok(())
    }

    #[test]
    fn test_add_word() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            project_config_path: Some(config_path.clone()),
            ..Default::default()
        };
        config.save()?;

        // Add a word
        config.add_word("testword")?;
        config.save()?;

        // Reload config and verify
        let loaded_config = load_from_file(ConfigType::Project, &config_path)?;
        assert!(loaded_config.is_allowed_word("testword"));

        Ok(())
    }

    #[test]
    fn test_add_word_global() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            global_config_path: Some(config_path.clone()),
            global_settings: RwLock::new(Some(ConfigSettings::default())),
            ..Default::default()
        };
        config.save_global()?;

        // Add a word
        config.add_word_global("testword")?;
        config.save_global()?;

        // Reload config and verify
        let loaded_config = load_from_file(ConfigType::Global, &config_path)?;
        assert!(loaded_config.is_allowed_word("testword"));

        Ok(())
    }

    #[test]
    fn test_ignore_patterns() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");
        let mut file = File::create(&config_path)?;
        let a = r#"
        ignore_patterns = [
            "^[ATCG]+$",
            "\\d{3}-\\d{2}-\\d{4}"  # Social Security Number format
        ]
        "#;
        file.write_all(a.as_bytes())?;

        let config = load_from_file(ConfigType::Project, &config_path)?;
        let patterns = config
            .effective_settings
            .read()
            .unwrap()
            .ignore_patterns
            .clone();
        assert!(patterns.contains(&String::from("^[ATCG]+$")));
        assert!(patterns.contains(&String::from("\\d{3}-\\d{2}-\\d{4}")));
        let reg = config.get_ignore_patterns();

        let patterns = reg.as_ref().unwrap();
        assert!(patterns.len() == 2);
        Ok(())
    }

    #[test]
    fn test_reload_ignore_patterns() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
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

        let config = load_from_file(ConfigType::Project, &config_path)?;
        assert!(config.get_ignore_patterns().unwrap().len() == 1);

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
        assert!(config.get_ignore_patterns().unwrap().len() == 2);

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
        assert!(config.get_ignore_patterns().unwrap().is_empty());

        Ok(())
    }

    #[test]
    fn test_config_recursive_search() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
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

        let config = CodebookConfig::load_configs(&sub_sub_dir)?;
        assert!(
            config
                .effective_settings
                .read()
                .unwrap()
                .words
                .contains(&"testword".to_string())
        );

        // Check that the config file path is stored
        assert_eq!(config.project_config_path, Some(config_path));
        Ok(())
    }

    #[test]
    fn test_should_ignore_path() {
        let config = CodebookConfig::default();
        config
            .effective_settings
            .write()
            .unwrap()
            .ignore_paths
            .push("target/**/*".to_string());

        assert!(config.should_ignore_path("target/debug/build"));
        assert!(!config.should_ignore_path("src/main.rs"));
    }

    #[test]
    fn test_reload() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            project_config_path: Some(config_path.clone()),
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
    fn test_reload_when_deleted() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            project_config_path: Some(config_path.clone()),
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
        config.reload()?;
        assert!(!config.is_allowed_word("testword"));

        Ok(())
    }

    #[test]
    fn test_add_word_case() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            project_config_path: Some(config_path.clone()),
            ..Default::default()
        };
        config.save()?;

        // Add a word with mixed case
        config.add_word("TestWord")?;
        config.save()?;

        // Reload config and verify with different cases
        let loaded_config = load_from_file(ConfigType::Global, &config_path)?;
        assert!(loaded_config.is_allowed_word("testword"));
        assert!(loaded_config.is_allowed_word("TESTWORD"));
        assert!(loaded_config.is_allowed_word("TestWord"));

        Ok(())
    }

    #[test]
    fn test_add_word_global_case() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        let config = CodebookConfig {
            global_config_path: Some(config_path.clone()),
            global_settings: RwLock::new(Some(ConfigSettings::default())),
            ..Default::default()
        };
        config.save_global()?;

        // Add a word with mixed case
        config.add_word_global("TestWord")?;
        config.save_global()?;

        // Reload config and verify with different cases
        let loaded_config = load_from_file(ConfigType::Global, &config_path)?;
        assert!(loaded_config.is_allowed_word("testword"));
        assert!(loaded_config.is_allowed_word("TESTWORD"));
        assert!(loaded_config.is_allowed_word("TestWord"));

        Ok(())
    }

    #[test]
    fn test_global_and_project_config() -> Result<(), io::Error> {
        // Create temporary directories for global and project configs
        let global_temp = TempDir::new().unwrap();
        let project_temp = TempDir::new().unwrap();

        // Set up global config path
        let global_config_dir = global_temp.path().join("codebook");
        fs::create_dir_all(&global_config_dir)?;
        let global_config_path = global_config_dir.join("codebook.toml");

        // Create global config with some settings
        let mut global_file = File::create(&global_config_path)?;
        write!(
            global_file,
            r#"
            dictionaries = ["en_US", "fr_FR"]
            words = ["globalword1", "globalword2"]
            flag_words = ["globaltodo"]
            "#
        )?;

        // Create project config with some different settings
        let project_config_path = project_temp.path().join("codebook.toml");
        let mut project_file = File::create(&project_config_path)?;
        write!(
            project_file,
            r#"
            words = ["projectword"]
            flag_words = ["projecttodo"]
            use_global = true
            "#
        )?;

        // Create a mock config with our test paths
        let config = CodebookConfig {
            global_config_path: Some(global_config_path),
            project_config_path: Some(project_config_path),
            ..Default::default()
        };

        // Manually load both configs to test merging
        if let Ok(global_settings) =
            CodebookConfig::load_settings_from_file(config.global_config_path.as_ref().unwrap())
        {
            *config.global_settings.write().unwrap() = Some(global_settings);
        }

        if let Ok(project_settings) =
            CodebookConfig::load_settings_from_file(config.project_config_path.as_ref().unwrap())
        {
            *config.project_settings.write().unwrap() = project_settings.clone();

            // Merge settings
            if project_settings.use_global {
                if let Some(global_settings) = config.global_settings.read().unwrap().as_ref() {
                    let mut effective = global_settings.clone();
                    effective.merge(project_settings);
                    *config.effective_settings.write().unwrap() = effective;
                } else {
                    *config.effective_settings.write().unwrap() = project_settings;
                }
            } else {
                *config.effective_settings.write().unwrap() = project_settings;
            }
        }

        // Verify merged results
        assert!(config.is_allowed_word("globalword1")); // From global
        assert!(config.is_allowed_word("projectword")); // From project
        assert!(config.should_flag_word("globaltodo")); // From global
        assert!(config.should_flag_word("projecttodo")); // From project

        // Verify dictionaries came from global
        let dictionaries = config.get_dictionary_ids();
        assert_eq!(dictionaries.len(), 2);
        assert!(dictionaries.contains(&"en_us".to_string()));
        assert!(dictionaries.contains(&"fr_fr".to_string()));

        // Now test with use_global = false
        let mut project_file = File::create(config.project_config_path.clone().unwrap())?;
        write!(
            project_file,
            r#"
            words = ["projectword"]
            flag_words = ["projecttodo"]
            use_global = false
            "#
        )?;

        // Reload
        config.reload()?;

        // Now should only see project words
        assert!(config.is_allowed_word("projectword")); // From project
        assert!(!config.is_allowed_word("globalword1")); // Not used from global
        assert!(config.should_flag_word("projecttodo")); // From project
        assert!(!config.should_flag_word("globaltodo")); // Not used from global

        Ok(())
    }
}
