use codebook::parser::{TextRange, WordLocation};
use std::collections::HashMap;

pub struct ExpectedMisspellingsResult {
    /// The full body of text to be used in spell checking, with
    /// the start/end delimiters removed.
    pub content: String,
    /// List of misspellings, sorted lexicographically by word.
    pub misspellings: Vec<WordLocation>,
}

/// Finds all words in the supplied `raw_content` between the specified
/// start/end delimiters. The words inside the delimiters represent words
/// that should be considered misspelled.
pub fn get_marked_misspellings(
    raw_content: &str,
    start_delimiter: &str,
    end_delimiter: &str,
) -> ExpectedMisspellingsResult {
    let mut cursor = 0;
    let mut content = String::new();
    let mut misspelled_words: HashMap<&str, WordLocation> = HashMap::new();
    let mut end_index = 0;

    // Find first instance of the start delimiter starting at the cursor.
    while let Some(start_offset) = raw_content[cursor..].find(start_delimiter) {
        let start_index = start_offset + cursor + start_delimiter.len();

        // Next, look for a matching end delimiter, exiting if there isn't one.
        let Some(end_offset) = raw_content[start_index..].find(end_delimiter) else {
            break;
        };

        end_index = start_index + end_offset;
        let word = &raw_content[start_index..end_index];

        // Compute the start and end bytes in the content string (not
        // the raw content, which has the extra delimiters).
        content += &raw_content[cursor..start_index - start_delimiter.len()];
        let start_byte = content.len();
        content += word;
        let end_byte = content.len();

        let range = TextRange {
            start_byte,
            end_byte,
        };

        if let Some(word_info) = misspelled_words.get_mut(word) {
            word_info.locations.push(range);
        } else {
            misspelled_words.insert(word, WordLocation::new(word.to_string(), vec![range]));
        }

        cursor = end_index + end_delimiter.len();
    }

    // Add rest of the raw content after the final end delimiter.
    content += &raw_content[end_index + end_delimiter.len()..];

    let mut misspellings: Vec<WordLocation> = misspelled_words.into_values().collect();
    misspellings.sort_by(|w1, w2| w1.word.cmp(&w2.word));

    ExpectedMisspellingsResult {
        content,
        misspellings,
    }
}

/// Checks that two sorted sequences of WordLocations are equal, panicking with
/// helpful debug information on failure.
#[macro_export]
macro_rules! assert_word_locations_match {
    ($actual:expr, $expected:expr) => {{
        let actual_val = $actual;
        let expected_val = $expected;
        let actual_words = actual_val.iter().map(|w| w.word.as_str());
        let expected_words = expected_val.iter().map(|w| w.word.as_str());

        // Warn the user if the lists of words are different.
        if actual_val.len() != expected_val.len() {
            panic!(
                "word list mismatch: actual.len() = {}, expected.len() = {}\n\nactual words = {:?}\n\nexpected words = {:?}",
                actual_val.len(),
                expected_val.len(),
                actual_words.collect::<Vec<&str>>(),
                expected_words.collect::<Vec<&str>>()
            );
        }

        // Otherwise, go word-by-word and fail if at the first error.
        for (i, (a, e)) in actual_val.iter().zip(expected_val.iter()).enumerate() {
            if a.word != e.word {
                panic!(
                    "word mismatch at index {}:\n actual = {:#?}\n expected = {:#?}\n\n",
                    i, a, e
                );
            }

            // Locations are not necessarily sorted by start byte, so
            // sort them before comparison.
            let mut a_loc = a.locations.clone();
            let mut e_loc = e.locations.clone();
            a_loc.sort_by(|l1, l2| l1.start_byte.cmp(&l2.start_byte));
            e_loc.sort_by(|l1, l2| l1.start_byte.cmp(&l2.start_byte));

            if a_loc  != e_loc {
                panic!(
                    "location mismatch for \"{}\" at index {}:\n actual = {:#?}\n expected = {:#?}",
                    a.word, i, a_loc, e_loc
                );
            }
        }
    }};
}

pub(crate) use assert_word_locations_match;

#[test]
fn test_get_expected_misspellings_simple() {
    let result = get_marked_misspellings(" ^a$ ^a$ a ^A$ ^^b$ ", "^", "$");

    assert_eq!(result.content, " a a a A ^b ");
    assert_eq!(
        result.misspellings,
        vec![
            WordLocation::new(
                "A".to_string(),
                vec![TextRange {
                    start_byte: 7,
                    end_byte: 8
                },]
            ),
            WordLocation::new(
                "^b".to_string(),
                vec![TextRange {
                    start_byte: 9,
                    end_byte: 11
                }]
            ),
            WordLocation::new(
                "a".to_string(),
                vec![
                    TextRange {
                        start_byte: 1,
                        end_byte: 2
                    },
                    TextRange {
                        start_byte: 3,
                        end_byte: 4
                    },
                ]
            ),
        ]
    );
}
