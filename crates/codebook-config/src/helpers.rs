use crate::settings::ConfigSettings;
use glob::Pattern;
use log::error;
use regex::Regex;
use std::path::Path;

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
