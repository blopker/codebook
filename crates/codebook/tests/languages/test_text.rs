use codebook::queries::LanguageType;

use super::utils::{assert_spelling_at, get_processor, spell_check};

#[test]
fn test_text_simple() {
    let sample_text = r#"
        I'm bvd at splellin Wolrd wolrd
        hello caféx regular regu
    "#;
    // "regu" also appears inside "regular" (occurrence 0); only the
    // standalone occurrence is flagged.
    assert_spelling_at(
        LanguageType::Text,
        sample_text,
        &[
            ("Wolrd", &[0]),
            ("bvd", &[0]),
            ("caféx", &[0]),
            ("regu", &[1]),
            ("splellin", &[0]),
            ("wolrd", &[0]),
        ],
    );
}

/// Anchor test for the Text path (no grammar: the whole input is
/// word-split). Multi-byte content — emoji, ZWJ sequences, accented chars —
/// sits BEFORE the misspellings so that any UTF-16/char-offset confusion in
/// range arithmetic shifts the reported ranges and fails the comparison.
/// Expected ranges are derived from the text, and `spell_check` in utils
/// additionally asserts every reported range slices back to its word.
#[test]
fn test_text_location_multibyte_anchor() {
    let sample_text = "café 👨‍👩‍👧‍👦 wrold, and 🌍 wrold again";
    assert_spelling_at(LanguageType::Text, sample_text, &[("wrold", &[0, 1])]);
}

#[test]
fn test_text_no_false_positives_after_emoji() {
    let processor = get_processor();
    let results = spell_check(
        &processor,
        LanguageType::Text,
        "hello 😀 world, this is all spelled correctly",
    );
    assert!(results.is_empty(), "unexpected flags: {results:?}");
}
