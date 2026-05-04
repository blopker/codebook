use std::sync::Arc;

use super::dictionary::Dictionary;

/// Maximum number of transliterable bigrams in a single word before we
/// give up generating variants. 4 bigrams = 15 variants; words with more
/// transliterable pairs are vanishingly rare and not worth the blowup.
const MAX_TRANSLITERABLE_PAIRS: usize = 4;

/// Hunspell `suggest` runs an edit-distance search and is expensive, so we
/// cap how many transliterated variants we feed it.
const MAX_SUGGEST_VARIANT_QUERIES: usize = 4;

/// Suggestion result cap; matches `HunspellDictionary::suggest`.
const MAX_SUGGESTIONS: usize = 5;

/// Selects which transliteration scheme a dictionary opts into.
/// Stored on `HunspellRepo` so language-specific knowledge lives next to the
/// dictionary definition rather than in the loader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transliteration {
    /// One-way ASCII → diacritic: `ae`/`oe`/`ue` → `ä`/`ö`/`ü`, lowercase `ss`
    /// → `ß`. Words already containing umlauts/eszett are unaffected — the
    /// dictionary stores the diacritic forms, so they hit on the direct check.
    German,
}

impl Transliteration {
    pub fn variants_fn(self) -> fn(&str) -> Vec<String> {
        match self {
            Transliteration::German => german_umlaut_variants,
        }
    }
}

/// Generate all umlaut/eszett variants of an ASCII-transliterated German word.
///
/// Maps `ae`/`oe`/`ue` → `ä`/`ö`/`ü` (preserving case) and lowercase `ss` → `ß`.
/// Each transliterable bigram independently may or may not be substituted, so the
/// result enumerates all 2^n − 1 non-empty combinations.
///
/// Returns an empty Vec when the word contains no transliterable bigrams or when
/// the count exceeds `MAX_TRANSLITERABLE_PAIRS`. Allocates nothing in those cases.
pub fn german_umlaut_variants(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.chars().collect();
    let mut positions: Vec<(usize, char)> = Vec::new();

    let mut i = 0;
    while i + 1 < chars.len() {
        let replacement = match (chars[i], chars[i + 1]) {
            ('a', 'e') => Some('ä'),
            ('A', 'e') | ('A', 'E') => Some('Ä'),
            ('o', 'e') => Some('ö'),
            ('O', 'e') | ('O', 'E') => Some('Ö'),
            ('u', 'e') => Some('ü'),
            ('U', 'e') | ('U', 'E') => Some('Ü'),
            ('s', 's') => Some('ß'),
            _ => None,
        };
        if let Some(r) = replacement {
            positions.push((i, r));
            if positions.len() > MAX_TRANSLITERABLE_PAIRS {
                return Vec::new();
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    if positions.is_empty() {
        return Vec::new();
    }

    let n = positions.len();
    let total = (1u32 << n) - 1;
    let mut variants = Vec::with_capacity(total as usize);

    for mask in 1u32..=total {
        let mut result = String::with_capacity(word.len());
        let mut idx = 0;
        let mut pos_idx = 0;
        while idx < chars.len() {
            if pos_idx < n && positions[pos_idx].0 == idx {
                if (mask >> pos_idx) & 1 == 1 {
                    result.push(positions[pos_idx].1);
                    idx += 2;
                } else {
                    result.push(chars[idx]);
                    idx += 1;
                }
                pos_idx += 1;
            } else {
                result.push(chars[idx]);
                idx += 1;
            }
        }
        variants.push(result);
    }

    variants
}

/// Wraps a Dictionary with reverse-transliteration on miss.
///
/// On `check`, first delegates to the inner dictionary. If that misses, generates
/// variants via `variants_fn` and returns true if any variant matches. The inner
/// dictionary's own check cache absorbs repeated lookups, so steady-state cost is
/// O(variants) hash lookups per miss and zero overhead per hit.
pub struct TransliteratingDictionary {
    inner: Arc<dyn Dictionary>,
    variants_fn: fn(&str) -> Vec<String>,
}

impl TransliteratingDictionary {
    pub fn new(inner: Arc<dyn Dictionary>, variants_fn: fn(&str) -> Vec<String>) -> Self {
        Self { inner, variants_fn }
    }
}

impl Dictionary for TransliteratingDictionary {
    fn check(&self, word: &str) -> bool {
        if self.inner.check(word) {
            return true;
        }
        (self.variants_fn)(word)
            .iter()
            .any(|v| self.inner.check(v))
    }

    fn suggest(&self, word: &str) -> Vec<String> {
        let variants = (self.variants_fn)(word);
        if variants.is_empty() {
            return self.inner.suggest(word);
        }

        // Querying the inner suggester with a transliterated variant gives it a
        // much better starting point than the original ASCII typo. Order results
        // by likelihood so the inner cap (MAX_SUGGESTIONS) doesn't drop the
        // useful entries:
        //   1. fully-substituted variant — most likely intended target
        //   2. the original word — catches non-transliteration typos
        //   3. partially-substituted variants — fall-back coverage
        let mut suggestions: Vec<String> = Vec::new();
        let push_unique = |dest: &mut Vec<String>, s: String| {
            if !dest.contains(&s) {
                dest.push(s);
            }
        };

        let last_idx = variants.len() - 1;
        for s in self.inner.suggest(&variants[last_idx]) {
            push_unique(&mut suggestions, s);
        }
        for s in self.inner.suggest(word) {
            push_unique(&mut suggestions, s);
        }
        let remaining_quota = MAX_SUGGEST_VARIANT_QUERIES.saturating_sub(1);
        for idx in (0..last_idx).rev().take(remaining_quota) {
            for s in self.inner.suggest(&variants[idx]) {
                push_unique(&mut suggestions, s);
            }
        }

        suggestions.truncate(MAX_SUGGESTIONS);
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::RwLock;

    #[test]
    fn no_variants_for_word_without_bigrams() {
        assert!(german_umlaut_variants("hello").is_empty());
        assert!(german_umlaut_variants("rust").is_empty());
        assert!(german_umlaut_variants("a").is_empty());
        assert!(german_umlaut_variants("").is_empty());
    }

    #[test]
    fn no_variants_for_word_already_containing_umlauts() {
        assert!(german_umlaut_variants("Bücher").is_empty());
        assert!(german_umlaut_variants("Straße").is_empty());
    }

    #[test]
    fn single_bigram_produces_one_variant() {
        let v = german_umlaut_variants("buecher");
        assert_eq!(v, vec!["bücher".to_string()]);
    }

    #[test]
    fn preserves_case_on_replacement() {
        assert_eq!(german_umlaut_variants("Buecher"), vec!["Bücher".to_string()]);
        assert_eq!(german_umlaut_variants("UEbel"), vec!["Übel".to_string()]);
        assert_eq!(german_umlaut_variants("Ueber"), vec!["Über".to_string()]);
    }

    #[test]
    fn ss_only_lowercase() {
        assert_eq!(german_umlaut_variants("Strasse"), vec!["Straße".to_string()]);
        // Uppercase SS stays as SS — Hunspell case folding handles uppercase variants.
        assert!(german_umlaut_variants("STRASSE").is_empty());
    }

    #[test]
    fn multiple_bigrams_enumerate_all_subsets() {
        // "aeoe" has 2 transliterable pairs → 3 variants.
        let v: HashSet<String> = german_umlaut_variants("aeoe").into_iter().collect();
        let expected: HashSet<String> = ["äoe", "aeö", "äö"]
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(v, expected);
    }

    #[test]
    fn overlapping_pairs_consume_left_to_right() {
        // "aee": 'ae' at 0 consumes 'a','e'; remaining 'e' has no partner.
        assert_eq!(german_umlaut_variants("aee"), vec!["äe".to_string()]);
    }

    #[test]
    fn rejects_blowup_above_cap() {
        // 5 transliterable pairs = above the cap → returns empty.
        let v = german_umlaut_variants("aeaeaeaeae");
        assert!(v.is_empty(), "should bail out above the cap, got {v:?}");
    }

    /// Minimal Dictionary stub for testing the wrapper without spinning up Hunspell.
    struct StubDict {
        words: HashSet<String>,
        suggestions: HashMap<String, Vec<String>>,
        check_calls: RwLock<usize>,
        suggest_calls: RwLock<Vec<String>>,
    }

    impl StubDict {
        fn new<I: IntoIterator<Item = &'static str>>(words: I) -> Self {
            Self {
                words: words.into_iter().map(String::from).collect(),
                suggestions: HashMap::new(),
                check_calls: RwLock::new(0),
                suggest_calls: RwLock::new(Vec::new()),
            }
        }

        fn with_suggestion(mut self, word: &str, suggestions: &[&str]) -> Self {
            self.suggestions.insert(
                word.to_string(),
                suggestions.iter().map(|s| s.to_string()).collect(),
            );
            self
        }
    }

    impl Dictionary for StubDict {
        fn check(&self, word: &str) -> bool {
            *self.check_calls.write().unwrap() += 1;
            self.words.contains(word)
        }
        fn suggest(&self, word: &str) -> Vec<String> {
            self.suggest_calls.write().unwrap().push(word.to_string());
            self.suggestions.get(word).cloned().unwrap_or_default()
        }
    }

    #[test]
    fn wrapper_passes_through_when_inner_matches() {
        let stub = Arc::new(StubDict::new(["Bücher"]));
        let wrapped = TransliteratingDictionary::new(stub.clone(), german_umlaut_variants);
        assert!(wrapped.check("Bücher"));
        // Only the inner check ran — no variants generated.
        assert_eq!(*stub.check_calls.read().unwrap(), 1);
    }

    #[test]
    fn wrapper_finds_word_via_transliterated_variant() {
        let stub = Arc::new(StubDict::new(["Bücher"]));
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        assert!(wrapped.check("Buecher"));
    }

    #[test]
    fn wrapper_returns_false_when_no_variant_matches() {
        let stub = Arc::new(StubDict::new(["Bücher"]));
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        assert!(!wrapped.check("xyz"));
        assert!(!wrapped.check("foobar"));
    }

    #[test]
    fn wrapper_does_not_call_inner_for_extra_variants_when_no_bigrams() {
        let stub = Arc::new(StubDict::new([]));
        let wrapped = TransliteratingDictionary::new(stub.clone(), german_umlaut_variants);
        wrapped.check("hello");
        // One call: the original word. No variants exist, so no extra calls.
        assert_eq!(*stub.check_calls.read().unwrap(), 1);
    }

    #[test]
    fn suggest_passes_through_inner_suggestions() {
        let stub = Arc::new(
            StubDict::new(["hello"]).with_suggestion("helo", &["hello", "halo"]),
        );
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        let result = wrapped.suggest("helo");
        assert_eq!(result, vec!["hello".to_string(), "halo".to_string()]);
    }

    #[test]
    fn suggest_no_extra_inner_calls_without_bigrams() {
        let stub = Arc::new(StubDict::new([]).with_suggestion("helo", &["hello"]));
        let wrapped = TransliteratingDictionary::new(stub.clone(), german_umlaut_variants);
        wrapped.suggest("helo");
        // Only one suggest call — no bigrams means no variant queries.
        let calls = stub.suggest_calls.read().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "helo");
    }

    #[test]
    fn suggest_queries_transliterated_variant() {
        // User typed "Buechrr" (typo). Hunspell suggest on the original gives
        // nothing useful, but suggest on the transliterated "Büchrr" finds
        // "Bücher".
        let stub = Arc::new(
            StubDict::new(["Bücher"])
                .with_suggestion("Buechrr", &[])
                .with_suggestion("Büchrr", &["Bücher"]),
        );
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        let result = wrapped.suggest("Buechrr");
        assert!(
            result.contains(&"Bücher".to_string()),
            "expected 'Bücher' in suggestions, got {result:?}"
        );
    }

    #[test]
    fn suggest_dedupes_across_inner_and_variant_queries() {
        // Same suggestion comes back from both queries — should appear once.
        let stub = Arc::new(
            StubDict::new([])
                .with_suggestion("Buecher", &["Bücher"])
                .with_suggestion("Bücher", &["Bücher"]),
        );
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        let result = wrapped.suggest("Buecher");
        let occurrences = result.iter().filter(|s| s == &"Bücher").count();
        assert_eq!(occurrences, 1, "expected dedupe, got {result:?}");
    }

    #[test]
    fn suggest_keeps_variant_results_when_inner_fills_cap() {
        // Regression: when inner.suggest(word) already returns MAX_SUGGESTIONS
        // entries, the fully-substituted variant's high-confidence suggestion
        // must still appear at the top — not get truncated off the end.
        let stub = Arc::new(
            StubDict::new([])
                .with_suggestion("Buecher", &["x1", "x2", "x3", "x4", "x5"])
                .with_suggestion("Bücher", &["correct"]),
        );
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        let result = wrapped.suggest("Buecher");
        assert!(
            result.contains(&"correct".to_string()),
            "variant suggestion was dropped: {result:?}"
        );
        assert_eq!(
            result.first().map(String::as_str),
            Some("correct"),
            "fully-substituted variant suggestion should rank first"
        );
    }

    #[test]
    fn suggest_truncates_to_cap() {
        // Inner returns 3 + variant returns 4 = 7 total before truncation.
        let stub = Arc::new(
            StubDict::new([])
                .with_suggestion("Buecher", &["a", "b", "c"])
                .with_suggestion("Bücher", &["d", "e", "f", "g"]),
        );
        let wrapped = TransliteratingDictionary::new(stub, german_umlaut_variants);
        let result = wrapped.suggest("Buecher");
        assert!(result.len() <= MAX_SUGGESTIONS, "got {result:?}");
        assert_eq!(result.len(), MAX_SUGGESTIONS);
    }

    #[test]
    fn suggest_caps_variant_query_count() {
        // 3 bigrams = 7 variants, but cap limits inner suggest calls to
        // MAX_SUGGEST_VARIANT_QUERIES + 1 (the original word).
        let stub = Arc::new(StubDict::new([]));
        let wrapped = TransliteratingDictionary::new(stub.clone(), german_umlaut_variants);
        wrapped.suggest("aeoeue");
        let calls = stub.suggest_calls.read().unwrap();
        assert_eq!(calls.len(), 1 + MAX_SUGGEST_VARIANT_QUERIES);
    }

    #[test]
    fn suggest_queries_fully_substituted_variant_first() {
        // When the cap forces selection, the all-substituted variant must be
        // queried (it's the most likely intended target).
        let stub = Arc::new(StubDict::new([]));
        let wrapped = TransliteratingDictionary::new(stub.clone(), german_umlaut_variants);
        wrapped.suggest("aeoeue");
        let calls = stub.suggest_calls.read().unwrap();
        // "äöü" is the fully-substituted form of "aeoeue".
        assert!(
            calls.contains(&"äöü".to_string()),
            "fully-substituted variant not queried; calls = {calls:?}"
        );
    }
}
