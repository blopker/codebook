//! Writing settings back into an existing config file without disturbing
//! the parts that didn't change: comments, blank lines, key order, and the
//! layout of arrays all survive a save.

use crate::settings::ConfigSettings;
use std::collections::{HashMap, HashSet};
use toml_edit::{Array, DocumentMut, Item, Value};

/// Rewrite `existing` (the file's current text, which parses as `old`) so it
/// holds `new`. Only top-level keys whose values differ between `old` and
/// `new` are touched. Returns `None` when `existing` can't be edited in place,
/// so the caller can fall back to writing `new` from scratch.
pub(crate) fn update_document(
    existing: &str,
    old: &ConfigSettings,
    new: &ConfigSettings,
) -> Option<String> {
    // Line endings are normalized to `\n` while editing and restored after.
    let crlf = existing.contains("\r\n");
    let mut doc: DocumentMut = existing.parse().ok()?;
    // Serializing both sides with the same serializer makes equal values
    // print identically, so changed keys can be found by comparing text.
    let old_doc: DocumentMut = toml::to_string(old).ok()?.parse().ok()?;
    let new_doc: DocumentMut = toml::to_string(new).ok()?.parse().ok()?;

    let mut keys: Vec<&str> = old_doc.iter().map(|(key, _)| key).collect();
    for (key, _) in new_doc.iter() {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }

    let root = doc.as_table_mut();
    for key in keys {
        let new_item = new_doc.get(key);
        if old_doc.get(key).map(Item::to_string) == new_item.map(Item::to_string) {
            continue;
        }
        let Some(new_item) = new_item else {
            root.remove(key);
            continue;
        };
        let Some(existing) = root.get_mut(key) else {
            root.insert(key, new_item.clone());
            continue;
        };
        match (existing, new_item) {
            (Item::Value(Value::Array(existing)), Item::Value(Value::Array(target))) => {
                sync_array(existing, target);
            }
            // Keep the value's surrounding whitespace and trailing comment.
            (Item::Value(existing), Item::Value(value)) => {
                let mut value = value.clone();
                *value.decor_mut() = existing.decor().clone();
                *existing = value;
            }
            (existing, _) => *existing = new_item.clone(),
        }
    }
    let updated = doc.to_string();
    if crlf {
        Some(updated.replace("\r\n", "\n").replace('\n', "\r\n"))
    } else {
        Some(updated)
    }
}

/// How an array element is identified when comparing arrays: its string
/// content, so `'word'` and `"word"` match, or its text for other values.
fn element_key(value: &Value) -> String {
    match value.as_str() {
        Some(s) => s.to_string(),
        None => {
            let mut value = value.clone();
            value.decor_mut().clear();
            value.to_string()
        }
    }
}

/// Make `existing` hold the elements of `target` while keeping the elements
/// it already has, with their formatting and comments, where they are. New
/// elements go before the next element `target` lists after them.
fn sync_array(existing: &mut Array, target: &Array) {
    let wanted: HashSet<String> = target.iter().map(element_key).collect();
    existing.retain(|value| wanted.contains(&element_key(value)));
    let present: HashSet<String> = existing.iter().map(element_key).collect();

    // New elements, keyed by the existing element they go before; the rest
    // come after everything.
    let mut before: HashMap<String, Vec<Value>> = HashMap::new();
    let mut pending: Vec<Value> = Vec::new();
    for value in target.iter() {
        let key = element_key(value);
        if !present.contains(&key) {
            pending.push(value.clone());
        } else if !pending.is_empty() {
            before.entry(key).or_default().append(&mut pending);
        }
    }
    if before.is_empty() && pending.is_empty() {
        return;
    }

    if existing.is_empty() {
        fill_empty_array(existing, pending);
        return;
    }

    // A new element is laid out like its neighbors: on its own line when
    // the array is multi-line, after `", "` when it's inline. The separator
    // is the first element's leading whitespace without any comment (a
    // comment after a comma is part of the next element's prefix, and
    // mustn't be copied).
    let first_prefix = raw(existing.get(0).and_then(|v| v.decor().prefix()));
    let separator = match first_prefix.rfind('\n') {
        Some(newline) => format!("\n{}", &first_prefix[newline + 1..]),
        None => " ".to_string(),
    };
    let fresh = |values: Vec<Value>| -> Vec<Value> {
        values
            .into_iter()
            .map(|mut value| {
                value.decor_mut().clear();
                value.decor_mut().set_prefix(separator.as_str());
                value
            })
            .collect()
    };

    let old: Vec<Value> = existing.iter().cloned().collect();
    let mut merged: Vec<Value> = Vec::with_capacity(old.len() + wanted.len());
    for mut value in old {
        if let Some(new) = before.remove(&element_key(&value)) {
            let mut new = fresh(new);
            // `value`'s prefix is the text between the previous comma (or
            // `[`) and `value`. A comment ending that first line belongs to
            // what came before, so it stays there, ahead of the new
            // elements; comment lines above `value` stay with `value`.
            let old_prefix = raw(value.decor().prefix());
            if old_prefix.contains('\n') {
                let (comment, rest) = split_line_comment(&old_prefix);
                new[0]
                    .decor_mut()
                    .set_prefix(format!("{comment}{separator}"));
                value.decor_mut().set_prefix(rest);
            } else {
                new[0].decor_mut().set_prefix(old_prefix.as_str());
                value.decor_mut().set_prefix(separator.as_str());
            }
            merged.extend(new);
        }
        merged.push(value);
    }

    let mut trailing = raw(Some(existing.trailing()));
    if !pending.is_empty() {
        let mut new = fresh(pending);
        // The text between the old last element and `]`: after its comma
        // when there is one, otherwise in the element's own suffix. A
        // comment ending its first line stays with the old last element;
        // the rest still comes last.
        let last = merged.last_mut().expect("the array isn't empty");
        let after = if existing.trailing_comma() {
            std::mem::take(&mut trailing)
        } else {
            let suffix = raw(last.decor().suffix());
            last.decor_mut().set_suffix("");
            suffix
        };
        let (comment, rest) = split_line_comment(&after);
        new[0]
            .decor_mut()
            .set_prefix(format!("{comment}{separator}"));
        if existing.trailing_comma() {
            trailing = rest.to_string();
        } else {
            new.last_mut()
                .expect("there are new elements")
                .decor_mut()
                .set_suffix(rest);
        }
        merged.extend(new);
    }

    existing.clear();
    for value in merged {
        existing.push_formatted(value);
    }
    existing.set_trailing(trailing);
}

/// Put `values` into an array that has no elements, keeping whatever is
/// between its brackets. `[]` becomes `["a", "b"]`; a multi-line array gets
/// one element per line, after any comments already there.
fn fill_empty_array(existing: &mut Array, values: Vec<Value>) {
    let inside = raw(Some(existing.trailing()));
    let (first_prefix, separator, trailing) = match inside.rfind('\n') {
        Some(last_newline) => {
            let indent = inside
                .lines()
                .rfind(|line| !line.trim().is_empty())
                .map(|line| &line[..line.len() - line.trim_start().len()])
                .unwrap_or("    ");
            let separator = format!("\n{indent}");
            existing.set_trailing_comma(true);
            (
                format!("{}{separator}", &inside[..last_newline]),
                separator,
                inside[last_newline..].to_string(),
            )
        }
        None => (String::new(), " ".to_string(), String::new()),
    };
    for (i, mut value) in values.into_iter().enumerate() {
        value.decor_mut().clear();
        let prefix = if i == 0 { &first_prefix } else { &separator };
        value.decor_mut().set_prefix(prefix.as_str());
        existing.push_formatted(value);
    }
    existing.set_trailing(trailing);
}

/// Split whitespace from around an array element into the comment ending
/// its first line, if there is one, and everything after.
fn split_line_comment(text: &str) -> (&str, &str) {
    match text.find('\n') {
        Some(newline) if text[..newline].trim_start().starts_with('#') => text.split_at(newline),
        _ => ("", text),
    }
}

fn raw(text: Option<&toml_edit::RawString>) -> String {
    text.and_then(|t| t.as_str()).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> ConfigSettings {
        toml::from_str(text).unwrap()
    }

    /// Apply `change` to the settings parsed from `existing` and return the
    /// updated file text.
    fn edit(existing: &str, change: impl FnOnce(&mut ConfigSettings)) -> String {
        let old = parse(existing);
        let mut new = old.clone();
        change(&mut new);
        let updated = update_document(existing, &old, &new).unwrap();
        // Whatever the layout, the result must parse back to `new`.
        assert_eq!(
            toml::to_string(&parse(&updated)).unwrap(),
            toml::to_string(&new).unwrap(),
            "updated text doesn't round-trip:\n{updated}"
        );
        updated
    }

    #[test]
    fn test_add_word_keeps_comments_and_layout() {
        let existing = r#"# Dictionaries to use for spell checking.
dictionaries = ["en_us"]

# Project specific terms dictionary.
words = [
  "myacronym"
]
"#;
        let updated = edit(existing, |s| {
            s.insert_word("customword");
        });
        assert_eq!(
            updated,
            r#"# Dictionaries to use for spell checking.
dictionaries = ["en_us"]

# Project specific terms dictionary.
words = [
  "customword",
  "myacronym"
]
"#
        );
    }

    #[test]
    fn test_add_word_appends_in_multiline_array_with_trailing_comma() {
        let existing = "words = [\n    \"alpha\", # first\n    \"beta\",\n]\n";
        let updated = edit(existing, |s| {
            s.insert_word("gamma");
        });
        assert_eq!(
            updated,
            "words = [\n    \"alpha\", # first\n    \"beta\",\n    \"gamma\",\n]\n"
        );
    }

    #[test]
    fn test_new_first_element_keeps_comment_after_bracket() {
        let existing = "words = [ # project terms\n  \"beta\",\n]\n";
        let updated = edit(existing, |s| {
            s.insert_word("alpha");
        });
        assert_eq!(
            updated,
            "words = [ # project terms\n  \"alpha\",\n  \"beta\",\n]\n"
        );
    }

    #[test]
    fn test_several_insertions_at_once() {
        let updated = edit("words = [\"beta\", \"delta\"]\n", |s| {
            for word in ["alpha", "gamma", "omega"] {
                s.insert_word(word);
            }
        });
        assert_eq!(
            updated,
            "words = [\"alpha\", \"beta\", \"delta\", \"gamma\", \"omega\"]\n"
        );
    }

    #[test]
    fn test_add_word_in_single_line_array() {
        let updated = edit("words = [\"alpha\", \"zeta\"]\n", |s| {
            s.insert_word("beta");
        });
        assert_eq!(updated, "words = [\"alpha\", \"beta\", \"zeta\"]\n");

        let updated = edit("words = [\"alpha\"]\n", |s| {
            s.insert_word("beta");
        });
        assert_eq!(updated, "words = [\"alpha\", \"beta\"]\n");

        let updated = edit("words = []\n", |s| {
            s.insert_word("beta");
        });
        assert_eq!(updated, "words = [\"beta\"]\n");
    }

    #[test]
    fn test_missing_key_is_added_before_tables() {
        let existing = r#"# My config
dictionaries = ["en_us"]

# Docs are prose
[[overrides]]
paths = ["docs/**"]
words = ["docword"]
"#;
        let updated = edit(existing, |s| {
            s.insert_ignore("target/**");
        });
        assert_eq!(
            updated,
            r#"# My config
dictionaries = ["en_us"]
ignore_paths = ["target/**"]

# Docs are prose
[[overrides]]
paths = ["docs/**"]
words = ["docword"]
"#
        );
    }

    #[test]
    fn test_unchanged_keys_and_defaults_are_not_written() {
        // Settings the file doesn't mention stay unmentioned, even though
        // serializing the settings from scratch would write their defaults.
        let existing = "words = [\"alpha\"] # trailing\n";
        let updated = edit(existing, |s| {
            s.insert_word("beta");
        });
        assert_eq!(updated, "words = [\"alpha\", \"beta\"] # trailing\n");
    }

    #[test]
    fn test_empty_file_gets_the_changed_key() {
        let updated = edit("", |s| {
            s.insert_word("alpha");
        });
        assert_eq!(updated, "words = [\"alpha\"]\n");
    }

    #[test]
    fn test_crlf_line_endings_are_kept() {
        let updated = edit("# words\r\nwords = [\r\n  \"alpha\",\r\n]\r\n", |s| {
            s.insert_word("beta");
        });
        assert_eq!(
            updated,
            "# words\r\nwords = [\r\n  \"alpha\",\r\n  \"beta\",\r\n]\r\n"
        );
    }

    #[test]
    fn test_append_without_trailing_comma() {
        let updated = edit("words = [\n  \"alpha\"\n]\n", |s| {
            s.insert_word("beta");
        });
        assert_eq!(updated, "words = [\n  \"alpha\",\n  \"beta\"\n]\n");
    }

    #[test]
    fn test_append_keeps_comment_on_last_element() {
        // Without a trailing comma, the comment is part of the last element.
        let updated = edit("words = [\n  \"alpha\",\n  \"beta\" # last\n]\n", |s| {
            s.insert_word("gamma");
        });
        assert_eq!(
            updated,
            "words = [\n  \"alpha\",\n  \"beta\", # last\n  \"gamma\"\n]\n"
        );

        // With one, the comment comes after the comma.
        let updated = edit("words = [\n  \"alpha\", # last\n]\n", |s| {
            s.insert_word("beta");
        });
        assert_eq!(updated, "words = [\n  \"alpha\", # last\n  \"beta\",\n]\n");
    }

    #[test]
    fn test_insert_keeps_comment_on_previous_element() {
        let updated = edit(
            "words = [\n  \"alpha\", # first\n  # about gamma\n  \"gamma\",\n]\n",
            |s| {
                s.insert_word("beta");
            },
        );
        assert_eq!(
            updated,
            "words = [\n  \"alpha\", # first\n  \"beta\",\n  # about gamma\n  \"gamma\",\n]\n"
        );
    }

    #[test]
    fn test_comment_above_first_element_stays_with_it() {
        let updated = edit("words = [\n  # section\n  \"beta\",\n]\n", |s| {
            s.insert_word("alpha");
        });
        assert_eq!(
            updated,
            "words = [\n  \"alpha\",\n  # section\n  \"beta\",\n]\n"
        );
    }

    #[test]
    fn test_add_word_in_empty_multiline_array() {
        let updated = edit("words = [\n]\n", |s| {
            s.insert_word("alpha");
        });
        assert_eq!(updated, "words = [\n    \"alpha\",\n]\n");

        let updated = edit("words = [\n  # none yet\n]\n", |s| {
            s.insert_word("alpha");
        });
        assert_eq!(updated, "words = [\n  # none yet\n  \"alpha\",\n]\n");
    }
}
