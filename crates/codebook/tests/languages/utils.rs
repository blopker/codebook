use std::path::PathBuf;
use std::sync::Arc;

use codebook::Codebook;
use codebook::parser::{TextRange, WordLocation};
use codebook::queries::LanguageType;
use codebook_config::{CodebookConfig, CodebookConfigMemory};

/// Build a Codebook that loads dictionaries from the checked-in fixtures
/// instead of downloading — tests must not touch the network (the downloader
/// is compiled with deny-network in test builds and would panic). Refresh
/// fixtures with `make fetch_fixtures`.
pub fn make_codebook(config: Arc<dyn CodebookConfig>) -> Codebook {
    init_logging();
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dictionaries");
    Codebook::with_dictionary_dir(config, Some(fixtures))
}

pub fn get_processor() -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config.add_ignore("**/ignore.txt");
    make_codebook(config)
}

pub fn get_processor_with_include(include: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config.add_include(include);
    make_codebook(config)
}

pub fn get_processor_with_include_and_ignore(include: &str, ignore: &str) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    config.add_include(include);
    config.add_ignore(ignore);
    make_codebook(config)
}

pub fn get_processor_with_tags(include_tags: Vec<&str>, exclude_tags: Vec<&str>) -> Codebook {
    let settings = codebook_config::ConfigSettings {
        include_tags: include_tags.into_iter().map(String::from).collect(),
        exclude_tags: exclude_tags.into_iter().map(String::from).collect(),
        ..Default::default()
    };
    let config = Arc::new(CodebookConfigMemory::new(settings));
    make_codebook(config)
}

pub fn init_logging() {
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Run spell_check and enforce the universal offset invariant: every reported
/// `TextRange`, sliced out of the source text, must equal the reported word.
/// This catches offset-arithmetic bugs (including multi-byte/emoji drift) in
/// every test, without any hand-written byte offsets.
pub fn spell_check(processor: &Codebook, lang: LanguageType, text: &str) -> Vec<WordLocation> {
    let results = processor.spell_check(text, Some(lang), None);
    for result in &results {
        for range in &result.locations {
            let slice = text.get(range.start_byte..range.end_byte).unwrap_or_else(|| {
                panic!(
                    "range {range:?} for word '{}' is out of bounds or splits a UTF-8 character",
                    result.word
                )
            });
            assert_eq!(
                slice, result.word,
                "range {range:?} for word '{}' slices to '{slice}'",
                result.word
            );
        }
    }
    results
}

fn flagged_words(results: &[WordLocation]) -> Vec<&str> {
    let mut words: Vec<&str> = results.iter().map(|r| r.word.as_str()).collect();
    words.sort_unstable();
    words
}

/// Assert the exact set of misspelled words in `text` — no more, no less.
///
/// Each expected word must occur exactly once in the sample text (as a
/// case-sensitive substring, since split words are flagged at sub-token
/// ranges). That makes word-set equality positional: a word flagged at the
/// wrong occurrence can't sneak through, without writing byte offsets. When a
/// test deliberately repeats a word to distinguish captured from uncaptured
/// nodes, use `assert_spelling_at` instead.
///
/// `not_flagged` is redundant with exact set equality but documents intent;
/// each entry must actually appear in the text so the guard exercises
/// something.
pub fn assert_spelling(lang: LanguageType, text: &str, expected: &[&str], not_flagged: &[&str]) {
    assert_spelling_with(&get_processor(), lang, text, expected, not_flagged);
}

pub fn assert_spelling_with(
    processor: &Codebook,
    lang: LanguageType,
    text: &str,
    expected: &[&str],
    not_flagged: &[&str],
) {
    for word in expected {
        let count = text.matches(word).count();
        assert_eq!(
            count, 1,
            "expected word '{word}' must occur exactly once in the sample text \
             (found {count}); use assert_spelling_at for deliberate repeats"
        );
    }
    for word in not_flagged {
        assert!(
            text.contains(word),
            "not_flagged word '{word}' does not appear in the sample text, so it tests nothing"
        );
    }

    let results = spell_check(processor, lang, text);
    let words = flagged_words(&results);
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        words, expected_sorted,
        "flagged words don't match expected set"
    );
}

/// Assert the exact set of misspelled words and, for each word, exactly which
/// occurrences are flagged — strict positional checking without hand-written
/// byte offsets.
///
/// Occurrence indices are 0-based positions in the byte-ordered list of
/// case-sensitive substring matches of the word in `text` (substring on
/// purpose: split words are flagged at sub-token ranges, e.g. `Userr` inside
/// `GetUserr`). A word flagged at occurrence 1 when occurrence 2 was expected
/// fails, exactly like a hand-maintained offset table would.
pub fn assert_spelling_at(lang: LanguageType, text: &str, expected: &[(&str, &[usize])]) {
    assert_spelling_at_with(&get_processor(), lang, text, expected);
}

pub fn assert_spelling_at_with(
    processor: &Codebook,
    lang: LanguageType,
    text: &str,
    expected: &[(&str, &[usize])],
) {
    let results = spell_check(processor, lang, text);
    let words = flagged_words(&results);
    let mut expected_words: Vec<&str> = expected.iter().map(|(w, _)| *w).collect();
    expected_words.sort_unstable();
    assert_eq!(
        words, expected_words,
        "flagged words don't match expected set"
    );

    for (word, indices) in expected {
        let matches: Vec<usize> = text.match_indices(word).map(|(i, _)| i).collect();
        for &i in *indices {
            assert!(
                i < matches.len(),
                "'{word}' occurs {} time(s) in the sample text, but occurrence \
                 index {i} was expected",
                matches.len()
            );
        }
        let mut expected_ranges: Vec<TextRange> = indices
            .iter()
            .map(|&i| TextRange {
                start_byte: matches[i],
                end_byte: matches[i] + word.len(),
            })
            .collect();
        expected_ranges.sort_unstable();

        let result = results.iter().find(|r| r.word == *word).unwrap();
        let mut actual_ranges = result.locations.clone();
        actual_ranges.sort_unstable();
        assert_eq!(
            actual_ranges, expected_ranges,
            "'{word}' flagged at the wrong occurrence(s): occurrences of the word \
             start at bytes {matches:?}, expected indices {indices:?}"
        );
    }
}

/// Proves the deny-network dev-dependency feature reaches the downloader in
/// this crate's test builds: a dictionary miss (cold cache, no fixture) must
/// panic instead of downloading.
#[test]
#[should_panic(expected = "Blocked network request")]
fn network_guard_active_in_test_builds() {
    use codebook::dictionaries::manager::DictionaryManager;
    let temp_cache = tempfile::tempdir().unwrap();
    let manager = DictionaryManager::new(&temp_cache.path().to_path_buf());
    let _ = manager.get_dictionary("en_gb");
}
