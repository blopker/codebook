mod helpers;
mod settings;
mod watched_file;
use crate::helpers::expand_tilde;
use crate::settings::ConfigSettings;
use crate::watched_file::WatchedFile;
use log::debug;
use log::info;
use regex::Regex;
use std::env;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

static CACHE_DIR: &str = "codebook";
static GLOBAL_CONFIG_FILE: &str = "codebook.toml";
static USER_CONFIG_FILES: [&str; 2] = ["codebook.toml", ".codebook.toml"];

/// The main trait for Codebook configuration.
pub trait CodebookConfig: Sync + Send + Debug {
    fn add_word(&self, word: &str) -> Result<bool, io::Error>;
    fn add_word_global(&self, word: &str) -> Result<bool, io::Error>;
    fn add_ignore(&self, file: &str) -> Result<bool, io::Error>;
    fn get_dictionary_ids(&self) -> Vec<String>;
    fn should_ignore_path(&self, path: &Path) -> bool;
    fn should_include_path(&self, path: &Path) -> bool;
    fn is_allowed_word(&self, word: &str) -> bool;
    fn should_flag_word(&self, word: &str) -> bool;
    fn get_ignore_patterns(&self) -> Option<Vec<Regex>>;
    fn get_min_word_length(&self) -> usize;
    fn cache_dir(&self) -> &Path;
}

/// Internal mutable state
#[derive(Debug)]
struct ConfigInner {
    /// Project-specific config file watcher
    project_config: WatchedFile<ConfigSettings>,
    /// Global config file watcher
    global_config: WatchedFile<ConfigSettings>,
    /// Current snapshot
    snapshot: Arc<ConfigSettings>,
    /// Compiled regex patterns cache
    regex_cache: Option<Vec<Regex>>,
}

#[derive(Debug)]
pub struct CodebookConfigFile {
    /// Single lock protecting all mutable state
    inner: RwLock<ConfigInner>,
    /// Directory for caching
    pub cache_dir: PathBuf,
}

impl Default for CodebookConfigFile {
    fn default() -> Self {
        let inner = ConfigInner {
            project_config: WatchedFile::new(None),
            global_config: WatchedFile::new(None),
            snapshot: Arc::new(ConfigSettings::default()),
            regex_cache: None,
        };

        Self {
            inner: RwLock::new(inner),
            cache_dir: helpers::default_cache_dir(),
        }
    }
}

impl CodebookConfigFile {
    /// Load configuration by searching for both global and project-specific configs
    pub fn load(current_dir: Option<&Path>) -> Result<Self, io::Error> {
        Self::load_with_global_config(current_dir, None)
    }

    /// Load configuration with an explicit global config override.
    pub fn load_with_global_config(
        current_dir: Option<&Path>,
        global_config_path: Option<PathBuf>,
    ) -> Result<Self, io::Error> {
        debug!("Initializing CodebookConfig");

        if let Some(current_dir) = current_dir {
            let current_dir = Path::new(current_dir);
            Self::load_configs(current_dir, global_config_path)
        } else {
            let current_dir = env::current_dir()?;
            Self::load_configs(&current_dir, global_config_path)
        }
    }

    /// Load both global and project configuration
    fn load_configs(
        start_dir: &Path,
        global_config_override: Option<PathBuf>,
    ) -> Result<Self, io::Error> {
        let config = Self::default();
        let mut inner = config.inner.write().unwrap();

        // First, try to load global config
        let global_config_path = match global_config_override {
            Some(path) => Some(path.to_path_buf()),
            None => Self::find_global_config_path(),
        };

        if let Some(global_path) = global_config_path {
            let global_config = WatchedFile::new(Some(global_path.clone()));

            if global_path.exists() {
                inner.global_config = global_config
                    .load(|path| {
                        Self::load_settings_from_file(path)
                            .map_err(|e| format!("Failed to load global config: {}", e))
                    })
                    .unwrap_or_else(|e| {
                        debug!("{}", e);
                        WatchedFile::new(Some(global_path.clone()))
                    });
                debug!("Loaded global config from {}", global_path.display());
            } else {
                info!("No global config found, using default");
                inner.global_config = global_config;
            }
        }

        // Then try to find and load project config
        if let Some(project_path) = Self::find_project_config(start_dir)? {
            debug!("Found project config at {}", project_path.display());
            let project_config = WatchedFile::new(Some(project_path.clone()));

            inner.project_config = project_config
                .load(|path| {
                    Self::load_settings_from_file(path)
                        .map_err(|e| format!("Failed to load project config: {}", e))
                })
                .unwrap_or_else(|e| {
                    debug!("{}", e);
                    WatchedFile::new(Some(project_path.clone()))
                });

            debug!("Loaded project config from {}", project_path.display());
        } else {
            info!("No project config found, using default");
            // Set path to start_dir if no config is found
            let default_path = start_dir.join(USER_CONFIG_FILES[0]);
            inner.project_config = WatchedFile::new(Some(default_path));
        }

        // Calculate initial effective settings
        let effective =
            Self::calculate_effective_settings(&inner.project_config, &inner.global_config);
        inner.snapshot = Arc::new(effective);

        drop(inner);
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
        if cfg!(windows)
            && let Some(config_dir) = dirs::config_dir()
        {
            return Some(config_dir.join("codebook").join(GLOBAL_CONFIG_FILE));
        }

        None
    }

    /// Find project configuration by searching up from the current directory
    fn find_project_config(start_dir: &Path) -> Result<Option<PathBuf>, io::Error> {
        let config_files = USER_CONFIG_FILES;

        // Start from the given directory and walk up to root
        let mut current_dir = Some(start_dir.to_path_buf());

        while let Some(dir) = current_dir {
            // Try each possible config filename in the current directory
            for config_name in &config_files {
                let config_path = dir.join(config_name);
                if config_path.is_file() {
                    return Ok(Some(config_path));
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

    /// Calculate the effective settings based on global and project settings
    fn calculate_effective_settings(
        project_config: &WatchedFile<ConfigSettings>,
        global_config: &WatchedFile<ConfigSettings>,
    ) -> ConfigSettings {
        let project = project_config
            .content()
            .cloned()
            .unwrap_or_else(ConfigSettings::default);

        if project.use_global {
            if let Some(global) = global_config.content() {
                let mut effective = global.clone();
                effective.merge(project);
                effective
            } else {
                project
            }
        } else {
            project
        }
    }

    /// Get current configuration snapshot (cheap to clone)
    fn snapshot(&self) -> Arc<ConfigSettings> {
        self.inner.read().unwrap().snapshot.clone()
    }

    /// Reload both global and project configurations, only reading files if they've changed
    pub fn reload(&self) -> Result<bool, io::Error> {
        let mut inner = self.inner.write().unwrap();
        let mut changed = false;

        // Check and reload global config if changed
        let (new_global, global_changed) = inner
            .global_config
            .clone()
            .reload_if_changed(|path| {
                Self::load_settings_from_file(path).map_err(|e| e.to_string())
            })
            .unwrap_or_else(|e| {
                debug!("Failed to reload global config: {}", e);
                (inner.global_config.clone(), false)
            });

        if global_changed {
            debug!("Global config reloaded");
            inner.global_config = new_global;
            changed = true;
        }

        // Check and reload project config if changed
        let (new_project, project_changed) = inner
            .project_config
            .clone()
            .reload_if_changed(|path| {
                Self::load_settings_from_file(path).map_err(|e| e.to_string())
            })
            .unwrap_or_else(|e| {
                debug!("Failed to reload project config: {}", e);
                (inner.project_config.clone(), false)
            });

        if project_changed {
            debug!("Project config reloaded");
            inner.project_config = new_project;
            changed = true;
        }

        // Recalculate effective settings if anything changed
        if changed {
            let effective =
                Self::calculate_effective_settings(&inner.project_config, &inner.global_config);
            inner.snapshot = Arc::new(effective);
            inner.regex_cache = None; // Invalidate regex cache
        }

        Ok(changed)
    }

    /// Save the project configuration to its file
    pub fn save(&self) -> Result<(), io::Error> {
        let inner = self.inner.read().unwrap();

        let project_config_path = match inner.project_config.path() {
            Some(path) => path.to_path_buf(),
            None => return Ok(()),
        };

        let settings = match inner.project_config.content() {
            Some(settings) => settings,
            None => return Ok(()),
        };

        let content = toml::to_string_pretty(settings).map_err(io::Error::other)?;
        info!(
            "Saving project configuration to {}",
            project_config_path.display()
        );
        fs::write(&project_config_path, content)
    }

    /// Save the global configuration to its file
    pub fn save_global(&self) -> Result<(), io::Error> {
        let inner = self.inner.read().unwrap();

        let global_config_path = match inner.global_config.path() {
            Some(path) => path.to_path_buf(),
            None => return Ok(()),
        };

        #[cfg(not(windows))]
        let global_config_path = match expand_tilde(&global_config_path) {
            Some(p) => p,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Failed to expand tilde in path: {}",
                        global_config_path.display()
                    ),
                ));
            }
        };

        let settings = match inner.global_config.content() {
            Some(settings) => settings,
            None => return Ok(()),
        };

        let content = toml::to_string_pretty(settings).map_err(io::Error::other)?;
        info!(
            "Saving global configuration to {}",
            global_config_path.display()
        );
        // Create parent directories if they don't exist
        if let Some(parent) = global_config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&global_config_path, content)
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

    /// Get path to project config if it exists
    pub fn project_config_path(&self) -> Option<PathBuf> {
        self.inner
            .read()
            .unwrap()
            .project_config
            .path()
            .map(|p| p.to_path_buf())
    }

    /// Get path to global config if it exists
    pub fn global_config_path(&self) -> Option<PathBuf> {
        self.inner
            .read()
            .unwrap()
            .global_config
            .path()
            .map(|p| p.to_path_buf())
    }

    fn rebuild_snapshot(inner: &mut ConfigInner) {
        let effective =
            Self::calculate_effective_settings(&inner.project_config, &inner.global_config);
        inner.snapshot = Arc::new(effective);
        inner.regex_cache = None;
    }

    fn update_project_settings<F>(&self, update: F) -> bool
    where
        F: FnOnce(&mut ConfigSettings) -> bool,
    {
        let mut inner = self.inner.write().unwrap();
        let mut settings = inner
            .project_config
            .content()
            .cloned()
            .unwrap_or_else(ConfigSettings::default);

        if !update(&mut settings) {
            return false;
        }

        inner.project_config = inner.project_config.clone().with_content_value(settings);
        Self::rebuild_snapshot(&mut inner);
        true
    }

    fn update_global_settings<F>(&self, update: F) -> bool
    where
        F: FnOnce(&mut ConfigSettings) -> bool,
    {
        let mut inner = self.inner.write().unwrap();
        let mut settings = inner
            .global_config
            .content()
            .cloned()
            .unwrap_or_else(ConfigSettings::default);

        if !update(&mut settings) {
            return false;
        }

        inner.global_config = inner.global_config.clone().with_content_value(settings);
        Self::rebuild_snapshot(&mut inner);
        true
    }
}

impl CodebookConfig for CodebookConfigFile {
    /// Add a word to the project configs allowlist
    fn add_word(&self, word: &str) -> Result<bool, io::Error> {
        Ok(self.update_project_settings(|settings| helpers::insert_word(settings, word)))
    }
    /// Add a word to the global configs allowlist
    fn add_word_global(&self, word: &str) -> Result<bool, io::Error> {
        Ok(self.update_global_settings(|settings| helpers::insert_word(settings, word)))
    }

    /// Add a file to the ignore list
    fn add_ignore(&self, file: &str) -> Result<bool, io::Error> {
        Ok(self.update_project_settings(|settings| helpers::insert_ignore(settings, file)))
    }

    /// Get dictionary IDs from effective configuration
    fn get_dictionary_ids(&self) -> Vec<String> {
        let snapshot = self.snapshot();
        helpers::dictionary_ids(&snapshot)
    }

    /// Check if a path is included based on the effective configuration
    fn should_include_path(&self, path: &Path) -> bool {
        let snapshot = self.snapshot();
        helpers::should_include_path(&snapshot, path)
    }

    /// Check if a path should be ignored based on the effective configuration
    fn should_ignore_path(&self, path: &Path) -> bool {
        let snapshot = self.snapshot();
        helpers::should_ignore_path(&snapshot, path)
    }

    /// Check if a word is in the effective allowlist
    fn is_allowed_word(&self, word: &str) -> bool {
        let snapshot = self.snapshot();
        helpers::is_allowed_word(&snapshot, word)
    }

    /// Check if a word should be flagged according to effective configuration
    fn should_flag_word(&self, word: &str) -> bool {
        let snapshot = self.snapshot();
        helpers::should_flag_word(&snapshot, word)
    }

    /// Get the list of user-defined ignore patterns
    fn get_ignore_patterns(&self) -> Option<Vec<Regex>> {
        let mut inner = self.inner.write().unwrap();
        if inner.regex_cache.is_none() {
            let regex_set = helpers::build_ignore_regexes(&inner.snapshot.ignore_patterns);
            inner.regex_cache = Some(regex_set);
        }

        inner.regex_cache.clone()
    }

    /// Get the minimum word length which should be checked
    fn get_min_word_length(&self) -> usize {
        helpers::min_word_length(&self.snapshot())
    }

    fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

#[derive(Debug)]
pub struct CodebookConfigMemory {
    settings: RwLock<ConfigSettings>,
    cache_dir: PathBuf,
}

impl Default for CodebookConfigMemory {
    fn default() -> Self {
        Self {
            settings: RwLock::new(ConfigSettings::default()),
            cache_dir: helpers::default_cache_dir(),
        }
    }
}

impl CodebookConfigMemory {
    pub fn new(settings: ConfigSettings) -> Self {
        Self {
            settings: RwLock::new(settings),
            cache_dir: helpers::default_cache_dir(),
        }
    }
}

impl CodebookConfigMemory {
    /// Get current configuration snapshot (cheap to clone)
    fn snapshot(&self) -> Arc<ConfigSettings> {
        Arc::new(self.settings.read().unwrap().clone())
    }
}

impl CodebookConfig for CodebookConfigMemory {
    fn add_word(&self, word: &str) -> Result<bool, io::Error> {
        let mut settings = self.settings.write().unwrap();
        Ok(helpers::insert_word(&mut settings, word))
    }

    fn add_word_global(&self, word: &str) -> Result<bool, io::Error> {
        self.add_word(word)
    }

    fn add_ignore(&self, file: &str) -> Result<bool, io::Error> {
        let mut settings = self.settings.write().unwrap();
        Ok(helpers::insert_ignore(&mut settings, file))
    }

    fn get_dictionary_ids(&self) -> Vec<String> {
        let snapshot = self.snapshot();
        helpers::dictionary_ids(&snapshot)
    }

    fn should_include_path(&self, path: &Path) -> bool {
        let snapshot = self.snapshot();
        helpers::should_include_path(&snapshot, path)
    }

    fn should_ignore_path(&self, path: &Path) -> bool {
        let snapshot = self.snapshot();
        helpers::should_ignore_path(&snapshot, path)
    }

    fn is_allowed_word(&self, word: &str) -> bool {
        let snapshot = self.snapshot();
        helpers::is_allowed_word(&snapshot, word)
    }

    fn should_flag_word(&self, word: &str) -> bool {
        let snapshot = self.snapshot();
        helpers::should_flag_word(&snapshot, word)
    }

    fn get_ignore_patterns(&self) -> Option<Vec<Regex>> {
        let snapshot = self.snapshot();
        Some(helpers::build_ignore_regexes(&snapshot.ignore_patterns))
    }

    fn get_min_word_length(&self) -> usize {
        helpers::min_word_length(&self.snapshot())
    }

    fn cache_dir(&self) -> &Path {
        &self.cache_dir
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
    ) -> Result<CodebookConfigFile, io::Error> {
        let config = CodebookConfigFile::default();
        let mut inner = config.inner.write().unwrap();

        match config_type {
            ConfigType::Project => {
                if let Ok(settings) = CodebookConfigFile::load_settings_from_file(&path) {
                    let mut project_config = WatchedFile::new(Some(path.as_ref().to_path_buf()));
                    project_config = project_config.with_content_value(settings);
                    inner.project_config = project_config;

                    // Recalculate effective settings
                    let effective = CodebookConfigFile::calculate_effective_settings(
                        &inner.project_config,
                        &inner.global_config,
                    );
                    inner.snapshot = Arc::new(effective);
                }
            }
            ConfigType::Global => {
                if let Ok(settings) = CodebookConfigFile::load_settings_from_file(&path) {
                    let mut global_config = WatchedFile::new(Some(path.as_ref().to_path_buf()));
                    global_config = global_config.with_content_value(settings);
                    inner.global_config = global_config;

                    // Recalculate effective settings
                    let effective = CodebookConfigFile::calculate_effective_settings(
                        &inner.project_config,
                        &inner.global_config,
                    );
                    inner.snapshot = Arc::new(effective);
                }
            }
        }

        drop(inner);
        Ok(config)
    }

    #[test]
    fn test_save_global_creates_directories() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let global_dir = temp_dir.path().join("deep").join("nested").join("dir");
        let config_path = global_dir.join("codebook.toml");

        // Create config with a path that doesn't exist yet
        // Create a config with the global path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            let mut global_config = WatchedFile::new(Some(config_path.clone()));
            global_config = global_config.with_content_value(ConfigSettings::default());
            inner.global_config = global_config;
        }

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
        // Create a config with the project path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            inner.project_config = WatchedFile::new(Some(config_path.clone()));
        }
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
        // Create a config with the global path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            let global_config = WatchedFile::new(Some(config_path.clone()));
            inner.global_config = global_config.with_content_value(ConfigSettings::default());
        }
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
        let patterns = config.snapshot().ignore_patterns.clone();
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

        let config = CodebookConfigFile::load_configs(&sub_sub_dir, None)?;
        assert!(config.snapshot().words.contains(&"testword".to_string()));

        // Check that the config file path is stored
        assert_eq!(config.project_config_path(), Some(config_path));
        Ok(())
    }

    #[test]
    fn test_global_config_override_is_used() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&workspace_dir)?;
        let custom_global_dir = temp_dir.path().join("global");
        fs::create_dir_all(&custom_global_dir)?;
        let override_path = custom_global_dir.join("codebook.toml");

        fs::write(
            &override_path,
            r#"
            words = ["customword"]
            "#,
        )?;

        let config = CodebookConfigFile::load_with_global_config(
            Some(workspace_dir.as_path()),
            Some(override_path.clone()),
        )?;

        assert_eq!(config.global_config_path(), Some(override_path));
        assert!(config.is_allowed_word("customword"));
        Ok(())
    }

    #[test]
    fn test_should_ignore_path() {
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            let mut settings = inner
                .project_config
                .content()
                .cloned()
                .unwrap_or_else(ConfigSettings::default);
            settings.ignore_paths.push("target/**/*".to_string());
            inner.project_config = inner.project_config.clone().with_content_value(settings);

            // Recalculate effective settings
            let effective = CodebookConfigFile::calculate_effective_settings(
                &inner.project_config,
                &inner.global_config,
            );
            inner.snapshot = Arc::new(effective);
        }

        assert!(config.should_ignore_path("target/debug/build".as_ref()));
        assert!(!config.should_ignore_path("src/main.rs".as_ref()));
    }

    #[test]
    fn test_reload() -> Result<(), io::Error> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("codebook.toml");

        // Create initial config
        // Create a config with the project path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            inner.project_config = WatchedFile::new(Some(config_path.clone()));
        }
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
        // Create a config with the project path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            inner.project_config = WatchedFile::new(Some(config_path.clone()));
        }
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
        // Create a config with the project path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            inner.project_config = WatchedFile::new(Some(config_path.clone()));
        }
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
        // Create a config with the global path set
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            let global_config = WatchedFile::new(Some(config_path.clone()));
            inner.global_config = global_config.with_content_value(ConfigSettings::default());
        }
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
        // Create a config with both paths
        let config = CodebookConfigFile::default();
        {
            let mut inner = config.inner.write().unwrap();
            inner.global_config = WatchedFile::new(Some(global_config_path.clone()));
            inner.project_config = WatchedFile::new(Some(project_config_path.clone()));
        }

        // Manually load both configs to test merging
        {
            let mut inner = config.inner.write().unwrap();
            if let Ok(global_settings) =
                CodebookConfigFile::load_settings_from_file(&global_config_path)
            {
                inner.global_config = inner
                    .global_config
                    .clone()
                    .with_content_value(global_settings);
            }
            if let Ok(project_settings) =
                CodebookConfigFile::load_settings_from_file(&project_config_path)
            {
                inner.project_config = inner
                    .project_config
                    .clone()
                    .with_content_value(project_settings);
            }

            // Recalculate effective settings after loading both configs
            let effective = CodebookConfigFile::calculate_effective_settings(
                &inner.project_config,
                &inner.global_config,
            );
            inner.snapshot = Arc::new(effective);
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
        let mut project_file = File::create(config.project_config_path().unwrap())?;
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
