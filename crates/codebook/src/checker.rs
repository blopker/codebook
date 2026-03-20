use std::collections::{HashMap, HashSet};

use crate::dictionaries::dictionary::Dictionary;
use crate::parser::{TextRange, WordLocation};
use codebook_config::CodebookConfig;

/// A candidate word extracted from a text node, with its position
/// in original-document byte offsets.
#[derive(Debug, Clone, PartialEq)]
pub struct WordCandidate {
    pub word: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Check if a word should be flagged based on config and dictionaries.
/// Returns true if the word is correct (should NOT be flagged).
fn is_word_correct(
    word: &str,
    dictionaries: &[std::sync::Arc<dyn Dictionary>],
    config: &dyn CodebookConfig,
) -> bool {
    if config.should_flag_word(word) {
        return false;
    }
    if word.len() < config.get_min_word_length() {
        return true;
    }
    if config.is_allowed_word(word) {
        return true;
    }
    dictionaries.iter().any(|dict| dict.check(word))
}

/// Check candidate words against dictionaries and config rules.
/// Returns WordLocations for misspelled words, grouping all locations
/// of the same word together.
pub fn check_words(
    candidates: &[WordCandidate],
    dictionaries: &[std::sync::Arc<dyn Dictionary>],
    config: &dyn CodebookConfig,
) -> Vec<WordLocation> {
    // Group misspelled candidates by word, deduplicating identical spans.
    // Only misspelled words are inserted, matching the old behavior where
    // the debug_assert caught query bugs producing duplicate misspelling locations.
    let mut word_positions: HashMap<&str, HashSet<TextRange>> = HashMap::new();
    for candidate in candidates {
        if is_word_correct(&candidate.word, dictionaries, config) {
            continue;
        }
        let location = TextRange {
            start_byte: candidate.start_byte,
            end_byte: candidate.end_byte,
        };
        let added = word_positions
            .entry(&candidate.word)
            .or_default()
            .insert(location);

        debug_assert!(
            added,
            "Two of the same locations found. Make a better query. Word: {}, Location: {:?}",
            candidate.word, location
        );
    }

    word_positions
        .into_iter()
        .map(|(word, positions)| WordLocation::new(word.to_string(), positions.into_iter().collect()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionaries::dictionary::TextDictionary;
    use std::sync::Arc;

    fn make_candidates(words: &[(&str, usize, usize)]) -> Vec<WordCandidate> {
        words
            .iter()
            .map(|(word, start, end)| WordCandidate {
                word: word.to_string(),
                start_byte: *start,
                end_byte: *end,
            })
            .collect()
    }

    #[test]
    fn test_check_words_flags_unknown() {
        let dict = Arc::new(TextDictionary::new("hello\nworld\n"));
        let config = Arc::new(codebook_config::CodebookConfigMemory::default());
        let candidates = make_candidates(&[("hello", 0, 5), ("wrld", 6, 10)]);
        let results = check_words(&candidates, &[dict], config.as_ref());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "wrld");
    }

    #[test]
    fn test_check_words_groups_locations() {
        let dict = Arc::new(TextDictionary::new("hello\n"));
        let config = Arc::new(codebook_config::CodebookConfigMemory::default());
        let candidates = make_candidates(&[("wrld", 0, 4), ("wrld", 10, 14)]);
        let results = check_words(&candidates, &[dict], config.as_ref());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].word, "wrld");
        assert_eq!(results[0].locations.len(), 2);
    }

    #[test]
    fn test_check_words_respects_min_length() {
        let dict = Arc::new(TextDictionary::new(""));
        let config = Arc::new(codebook_config::CodebookConfigMemory::default());
        // Default min word length is 3
        let candidates = make_candidates(&[("ab", 0, 2)]);
        let results = check_words(&candidates, &[dict], config.as_ref());
        assert!(results.is_empty(), "Short words should be skipped");
    }

    #[test]
    fn test_check_words_respects_allowed_words() {
        let dict = Arc::new(TextDictionary::new(""));
        let config = Arc::new(codebook_config::CodebookConfigMemory::default());
        config.add_word("codebook").unwrap();
        let candidates = make_candidates(&[("codebook", 0, 8)]);
        let results = check_words(&candidates, &[dict], config.as_ref());
        assert!(results.is_empty(), "Allowed words should not be flagged");
    }
}
