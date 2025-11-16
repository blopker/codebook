use crate::settings::ConfigSettings;
use glob::Pattern;
use log::error;
use regex::Regex;
use std::env;
use std::path::{Path, PathBuf};

pub(crate) fn default_cache_dir() -> PathBuf {
    #[cfg(windows)]
    {
        windows_cache_dir()
    }

    #[cfg(not(windows))]
    {
        unix_cache_dir()
    }
}

#[cfg(windows)]
pub(crate) fn windows_cache_dir() -> PathBuf {
    if let Some(dir) = dirs::data_local_dir() {
        return dir.join("codebook").join("cache");
    }

    if let Some(dir) = dirs::data_dir() {
        return dir.join("codebook").join("cache");
    }

    if let Some(home) = dirs::home_dir() {
        return home
            .join("AppData")
            .join("Local")
            .join("codebook")
            .join("cache");
    }

    env::temp_dir().join("codebook").join("cache")
}

#[cfg(not(windows))]
pub(crate) fn unix_cache_dir() -> PathBuf {
    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME")
        && !xdg_data_home.is_empty()
    {
        return PathBuf::from(xdg_data_home).join("codebook").join("cache");
    }

    if let Some(home) = dirs::home_dir() {
        return home
            .join(".local")
            .join("share")
            .join("codebook")
            .join("cache");
    }

    env::temp_dir().join("codebook").join("cache")
}

/// Insert a word into the allowlist, returning true when it was newly added.
pub(crate) fn insert_word(settings: &mut ConfigSettings, word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    if settings.words.contains(&word) {
        return false;
    }
    settings.words.push(word);
    settings.words.sort();
    settings.words.dedup();
    true
}

/// Insert a path into the ignore list, returning true when it was newly added.
pub(crate) fn insert_ignore(settings: &mut ConfigSettings, file: &str) -> bool {
    let file = file.to_string();
    if settings.ignore_paths.contains(&file) {
        return false;
    }
    settings.ignore_paths.push(file);
    settings.ignore_paths.sort();
    settings.ignore_paths.dedup();
    true
}

/// Resolve configured dictionary IDs, providing a default when none are set.
pub(crate) fn dictionary_ids(settings: &ConfigSettings) -> Vec<String> {
    if settings.dictionaries.is_empty() {
        vec!["en_us".to_string()]
    } else {
        settings.dictionaries.clone()
    }
}

/// Determine whether a path should be ignored based on the configured glob patterns.
pub(crate) fn should_ignore_path(settings: &ConfigSettings, path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    settings.ignore_paths.iter().any(|pattern| {
        Pattern::new(pattern)
            .map(|p| p.matches(&path_str))
            .unwrap_or(false)
    })
}

/// Check if a word is explicitly allowed.
pub(crate) fn is_allowed_word(settings: &ConfigSettings, word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    settings.words.iter().any(|w| w == &word)
}

/// Check if a word should be flagged.
pub(crate) fn should_flag_word(settings: &ConfigSettings, word: &str) -> bool {
    let word = word.to_ascii_lowercase();
    settings.flag_words.iter().any(|w| w == &word)
}

/// Compile user-provided ignore regex patterns, dropping invalid entries.
pub(crate) fn build_ignore_regexes(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|pattern| match Regex::new(pattern) {
            Ok(regex) => Some(regex),
            Err(e) => {
                error!("Ignoring invalid regex pattern '{pattern}': {e}");
                None
            }
        })
        .collect()
}

/// Retrieve the configured minimum word length.
pub(crate) fn min_word_length(settings: &ConfigSettings) -> usize {
    settings.min_word_length
}

pub(crate) fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs::home_dir();
    }
    dirs::home_dir().map(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard};

    #[cfg(not(windows))]
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[cfg(not(windows))]
    fn lock_env_and_set_xdg(value: Option<&str>) -> (MutexGuard<'static, ()>, Option<OsString>) {
        let guard = ENV_MUTEX.lock().unwrap();
        let previous = env::var_os("XDG_DATA_HOME");

        unsafe {
            match value {
                Some(val) => env::set_var("XDG_DATA_HOME", val),
                None => env::remove_var("XDG_DATA_HOME"),
            }
        }

        (guard, previous)
    }

    #[cfg(not(windows))]
    fn restore_xdg(previous: Option<OsString>) {
        unsafe {
            match previous {
                Some(val) => env::set_var("XDG_DATA_HOME", val),
                None => env::remove_var("XDG_DATA_HOME"),
            }
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn unix_cache_dir_uses_xdg_data_home() {
        let (guard, previous) = lock_env_and_set_xdg(Some("/tmp/codebook-xdg"));

        let expected = PathBuf::from("/tmp/codebook-xdg")
            .join("codebook")
            .join("cache");

        assert_eq!(unix_cache_dir(), expected);

        restore_xdg(previous);
        drop(guard);
    }

    #[cfg(not(windows))]
    #[test]
    fn unix_cache_dir_falls_back_to_home() {
        let (guard, previous) = lock_env_and_set_xdg(Some(""));

        let home = dirs::home_dir().expect("home directory must be available for the test");
        let expected = home
            .join(".local")
            .join("share")
            .join("codebook")
            .join("cache");

        assert_eq!(unix_cache_dir(), expected);

        restore_xdg(previous);
        drop(guard);
    }

    #[cfg(not(windows))]
    #[test]
    fn default_cache_dir_matches_unix_on_non_windows() {
        let (guard, previous) = lock_env_and_set_xdg(Some("/tmp/codebook-xdg-default"));

        assert_eq!(default_cache_dir(), unix_cache_dir());

        restore_xdg(previous);
        drop(guard);
    }
}
