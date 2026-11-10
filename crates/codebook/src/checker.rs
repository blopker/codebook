use std::collections::{HashMap, HashSet};

use crate::dictionaries::dictionary::Dictionary;
use crate::parser::{TextRange, WordLocation};
use codebook_config::ConfigSettings;
use unicase::UniCase;

/// A candidate word extracted from a text node, with its position
/// in original-document byte offsets. Borrows the word text from the
/// source document to avoid per-word String allocations.
#[derive(Debug, Clone, PartialEq)]
pub struct WordCandidate<'a> {
    pub word: &'a str,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Check candidate words against dictionaries and the (already resolved)
/// settings for the file. Returns WordLocations for misspelled words,
/// grouping all locations of the same word together.
pub fn check_words(
    candidates: &[WordCandidate<'_>],
    dictionaries: &[std::sync::Arc<dyn Dictionary>],
    settings: &ConfigSettings,
) -> Vec<WordLocation> {
    // Group candidates by word text.
    let mut word_positions: HashMap<&str, Vec<TextRange>> = HashMap::new();
    for candidate in candidates {
        word_positions
            .entry(candidate.word)
            .or_default()
            .push(TextRange {
                start_byte: candidate.start_byte,
                end_byte: candidate.end_byte,
            });
    }

    // Index the word lists once so each lookup is a hash probe instead of a
    // scan of the whole list; allowlists grow to hundreds of entries.
    let flagged = fold_words(&settings.flag_words);
    let allowed = fold_words(&settings.words);
    let min_word_length = settings.min_word_length();

    // Check each unique word once
    let mut results = Vec::new();
    for (word, mut positions) in word_positions {
        // Identical spans (a query capturing the same text twice) collapse
        // to one location.
        let found = positions.len();
        positions.sort_unstable();
        positions.dedup();
        debug_assert!(
            positions.len() == found,
            "Two of the same locations found. Make a better query. Word: {word}, Locations: {positions:?}"
        );
        let folded = UniCase::new(word);
        if flagged.contains(&folded) {
            results.push(WordLocation::new(word.to_string(), positions));
            continue;
        }
        // Compare characters, not bytes: multi-byte scripts (Cyrillic, Greek,
        // accented Latin) would otherwise pass the length filter early.
        if word.chars().count() < min_word_length {
            continue;
        }
        if allowed.contains(&folded) {
            continue;
        }
        let is_correct = dictionaries.iter().any(|dict| dict.check(word));
        if !is_correct {
            results.push(WordLocation::new(word.to_string(), positions));
        }
    }
    results
}

fn fold_words(words: &[String]) -> HashSet<UniCase<&str>> {
    words.iter().map(|w| UniCase::new(w.as_str())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionaries::dictionary::TextDictionary;
    use std::sync::Arc;

    fn make_candidates<'a>(words: &[(&'a str, usize, usize)]) -> Vec<WordCandidate<'a>> {
        words
            .iter()
            .map(|(word, start, end)| WordCandidate {
                word,
                start_byte: *start,
                end_byte: *end,
            })
            .collect()
    }

    #[test]
    fn test_check_words_flags_unknown() {
        let dict = Arc::new(TextDictionary::new("hello\nworld\n"));
        let settings = ConfigSettings::default();
        let candidates = make_candidates(&[("hello", 0, 5), ("wrld", 6, 10)]);
        let results = check_words(&candidates, &[dict], &settings);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "wrld");
    }

    #[test]
    fn test_check_words_groups_locations() {
        let dict = Arc::new(TextDictionary::new("hello\n"));
        let settings = ConfigSettings::default();
        let candidates = make_candidates(&[("wrld", 0, 4), ("wrld", 10, 14)]);
        let results = check_words(&candidates, &[dict], &settings);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "wrld");
        assert_eq!(results[0].locations.len(), 2);
    }

    #[test]
    fn test_check_words_respects_min_length() {
        let dict = Arc::new(TextDictionary::new(""));
        let settings = ConfigSettings::default();
        // Default min word length is 3
        let candidates = make_candidates(&[("ab", 0, 2)]);
        let results = check_words(&candidates, &[dict], &settings);
        assert!(results.is_empty(), "Short words should be skipped");
    }

    #[test]
    fn test_check_words_min_length_counts_chars_not_bytes() {
        let dict = Arc::new(TextDictionary::new(""));
        let settings = ConfigSettings::default();
        // "ий" is 2 chars but 4 bytes; must be skipped like a 2-char ASCII word
        let candidates = make_candidates(&[("ий", 0, 4)]);
        let results = check_words(&candidates, &[dict], &settings);
        assert!(
            results.is_empty(),
            "Multi-byte short words should be skipped"
        );
    }

    #[test]
    fn test_check_words_respects_allowed_words() {
        let dict = Arc::new(TextDictionary::new(""));
        let mut settings = ConfigSettings::default();
        settings.insert_word("codebook");
        let candidates = make_candidates(&[("codebook", 0, 8)]);
        let results = check_words(&candidates, &[dict], &settings);
        assert!(results.is_empty(), "Allowed words should not be flagged");
    }
}
