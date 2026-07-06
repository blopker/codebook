use log::error;
use regex::{Regex, RegexBuilder};
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

/// Compile user-provided ignore regex patterns, dropping invalid entries.
/// Patterns are compiled with multiline mode so `^` and `$` match line boundaries.
pub fn build_ignore_regexes(patterns: &[String]) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(
            |pattern| match RegexBuilder::new(pattern).multi_line(true).build() {
                Ok(regex) => Some(regex),
                Err(e) => {
                    error!("Ignoring invalid regex pattern '{pattern}': {e}");
                    None
                }
            },
        )
        .collect()
}

/// Expand `~` and `~/` prefixes to the current user's home directory on all
/// platforms (`~\` is also accepted on Windows, where it is a separator).
/// Other paths, including `~user/...`, are returned unchanged.
pub(crate) fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();
    let path = p.to_string_lossy();

    if path == "~" {
        return dirs::home_dir();
    }

    let rest = path.strip_prefix("~/");
    #[cfg(windows)]
    let rest = rest.or_else(|| path.strip_prefix("~\\"));

    match rest {
        // Trim any extra leading separators (e.g. `~//x`): join with an
        // absolute remainder would replace the home directory entirely.
        Some(rest) => {
            dirs::home_dir().map(|home| home.join(rest.trim_start_matches(std::path::is_separator)))
        }
        None => Some(p.to_path_buf()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(windows))]
    use std::ffi::OsString;
    #[cfg(not(windows))]
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

    #[test]
    fn expand_tilde_resolves_home_directory() {
        let home = dirs::home_dir().expect("home directory must be available for the test");

        assert_eq!(expand_tilde("~"), Some(home));
    }

    #[test]
    fn expand_tilde_resolves_unix_style_home_path() {
        let home = dirs::home_dir().expect("home directory must be available for the test");

        assert_eq!(
            expand_tilde("~/dotfiles/codebook.toml"),
            Some(home.join("dotfiles/codebook.toml"))
        );
    }

    #[test]
    fn expand_tilde_stays_within_home_on_extra_separators() {
        let home = dirs::home_dir().expect("home directory must be available for the test");

        assert_eq!(expand_tilde("~//etc/passwd"), Some(home.join("etc/passwd")));
    }

    #[cfg(windows)]
    #[test]
    fn expand_tilde_resolves_windows_style_home_path() {
        let home = dirs::home_dir().expect("home directory must be available for the test");

        assert_eq!(
            expand_tilde(r"~\dotfiles\codebook.toml"),
            Some(home.join(r"dotfiles\codebook.toml"))
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn expand_tilde_leaves_windows_style_path_unchanged_on_unix() {
        // On Unix `\` is an ordinary filename character, not a separator.
        let path = PathBuf::from(r"~\dotfiles\codebook.toml");

        assert_eq!(expand_tilde(&path), Some(path));
    }

    #[test]
    fn expand_tilde_leaves_other_paths_unchanged() {
        let path = PathBuf::from("~user/codebook.toml");

        assert_eq!(expand_tilde(&path), Some(path));
    }

    #[test]
    fn test_build_ignore_regexes_valid_patterns() {
        let patterns = vec![r"\b[A-Z]{2,}\b".to_string(), r"TODO:.*".to_string()];

        let compiled = build_ignore_regexes(&patterns);
        assert_eq!(compiled.len(), 2);
        assert!(compiled[0].is_match("HTML"));
        assert!(compiled[1].is_match("TODO: fix this"));
    }

    #[test]
    fn test_build_ignore_regexes_invalid_pattern_skipped() {
        let patterns = vec![
            r"valid.*".to_string(),
            r"[invalid".to_string(), // Missing closing bracket
            r"also_valid".to_string(),
        ];

        let compiled = build_ignore_regexes(&patterns);
        // Invalid pattern should be skipped, not crash
        assert_eq!(compiled.len(), 2);
    }

    #[test]
    fn test_build_ignore_regexes_multiline_mode() {
        let patterns = vec![r"^vim\..*".to_string()];
        let compiled = build_ignore_regexes(&patterns);

        let text = "let x = 1\nvim.opt.showmode = false\nlet y = 2";

        // Should match line starting with vim. (multiline mode)
        assert!(compiled[0].is_match(text));

        let m = compiled[0].find(text).unwrap();
        assert_eq!(m.as_str(), "vim.opt.showmode = false");
    }
}
