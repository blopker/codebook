use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_markdown_paragraph() {
    assert_spelling(
        LanguageType::Markdown,
        "Some paragraph text with a misspeled word.\n",
        &["misspeled"],
        &[],
    );
}

#[test]
fn test_markdown_heading() {
    assert_spelling(
        LanguageType::Markdown,
        "# A headng with a tyypo\n",
        &["headng", "tyypo"],
        &[],
    );
}

#[test]
fn test_markdown_fenced_code_block_known_lang() {
    // bash.scm only captures comments, strings, function names, heredocs,
    // and variable names, NOT command invocations. So mkdir/some_dir are
    // not checked because bash.scm doesn't capture them, not because
    // they're in a bash dictionary.
    let sample_text = r#"# Hello World

Some correct text here.

```bash
mkdir some_dir
```

More correct text here.
"#;
    assert_spelling(LanguageType::Markdown, sample_text, &[], &["mkdir", "dir"]);
}

#[test]
fn test_markdown_fenced_code_block_unknown_lang_skipped() {
    let sample_text = r#"Some text.

```unknownlang
badwwword_in_code
```

More text.
"#;
    // Unknown language code blocks are completely skipped.
    assert_spelling(LanguageType::Markdown, sample_text, &[], &["badwwword"]);
}

#[test]
fn test_markdown_fenced_code_block_no_lang_skipped() {
    let sample_text = r#"Some text.

```
badwwword_in_code
```

More text.
"#;
    // Code blocks without language info are completely skipped.
    assert_spelling(LanguageType::Markdown, sample_text, &[], &["badwwword"]);
}

#[test]
fn test_markdown_code_block_uses_language_grammar() {
    // In Python grammar, function names are checked as identifiers, so the
    // typo inside the code block is flagged alongside both prose typos.
    let sample_text = r#"A paragrap with a tyypo.

```python
def some_functin():
    pass
```

Another paragrap with a tyypo.
"#;
    assert_spelling_at(
        LanguageType::Markdown,
        sample_text,
        &[("paragrap", &[0, 1]), ("tyypo", &[0, 1]), ("functin", &[0])],
    );
}

#[test]
fn test_markdown_multiple_code_blocks() {
    let sample_text = r#"Some text with a tyypo.

```bash
mkdir somedir
```

Middle text is corect.

```unknownlang
badspel = True
```

End text is also corect.
"#;
    // Exact set equality: bash commands (mkdir/somedir) aren't captured by
    // bash.scm and the unknown-language block (badspel) is skipped entirely,
    // so neither can appear in the flagged set.
    assert_spelling_at(
        LanguageType::Markdown,
        sample_text,
        &[("tyypo", &[0]), ("corect", &[0, 1])],
    );
}

#[test]
fn test_markdown_block_quote() {
    assert_spelling(
        LanguageType::Markdown,
        "> A block quoet with a tyypo.\n",
        &["quoet", "tyypo"],
        &[],
    );
}

#[test]
fn test_markdown_code_block_alias_resolution() {
    // Common aliases resolve to their language (py -> Python, js ->
    // Javascript), so "wrld" is flagged in both code blocks.
    let sample_text = r#"Some text.

```py
def hello_wrld():
    pass
```

```js
function hello_wrld() {}
```

More text.
"#;
    assert_spelling_at(LanguageType::Markdown, sample_text, &[("wrld", &[0, 1])]);
}

/// Anchor test for the injection path: ranges reported from an injected
/// language (python inside a markdown code block) must map back to original
/// document coordinates. The emoji in the prose sit BEFORE the code block so
/// any remapping that counts chars or UTF-16 units instead of UTF-8 bytes
/// shifts the ranges and fails the comparison. Expected ranges are derived
/// from the text, and `spell_check` in utils additionally asserts every
/// reported range slices back to its word.
#[test]
fn test_markdown_injected_region_offsets_multibyte_anchor() {
    let sample_text =
        "# OK 🎉\n\nProse 👨‍👩‍👧‍👦 with a tyypo.\n\n```python\ndef some_functin(): pass\n```\n";
    assert_spelling_at(
        LanguageType::Markdown,
        sample_text,
        &[("tyypo", &[0]), ("functin", &[0])],
    );
}

#[test]
fn test_markdown_no_duplicate_spans() {
    // Block quotes contain paragraphs. Make sure the inline content isn't
    // captured twice (once for the paragraph, once for the block quote):
    // a duplicated span would show up as an extra, unexpected range.
    assert_spelling_at(
        LanguageType::Markdown,
        "> A tyypo in a block quoet.\n",
        &[("tyypo", &[0]), ("quoet", &[0])],
    );
}
