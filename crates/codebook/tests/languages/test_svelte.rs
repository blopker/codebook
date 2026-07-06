use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_svelte_simple() {
    let sample_text = r#"<script>
    // A componet with speling erors
    let couter = 0;
    function incremnt() {
        couter += 1;
    }
</script>

<h1>Welcom to my aplicaton</h1>
<p>The curent count is {couter}</p>
<button on:click={incremnt}>Incrementt</button>

<style>
    .containr {
        color: red;
    }
</style>
"#;
    assert_spelling_at(
        LanguageType::HTML,
        sample_text,
        &[
            // Comment in the script block.
            ("componet", &[0]),
            ("speling", &[0]),
            ("erors", &[0]),
            // Flagged at `let couter` (definition) and inside `{couter}` in
            // the <p> element, which is plain HTML text content; the
            // `couter += 1` usage is not flagged.
            ("couter", &[0, 2]),
            // Flagged at the function definition; the `on:click={incremnt}`
            // attribute value is not.
            ("incremnt", &[0]),
            // HTML text content.
            ("Welcom", &[0]),
            ("aplicaton", &[0]),
            ("curent", &[0]),
            ("Incrementt", &[0]),
            // CSS class identifier in the style block.
            ("containr", &[0]),
        ],
    );
}

/// Passing a .svelte file path routes the text through the HTML/Svelte
/// pipeline; needs a raw processor call because the shared helpers don't
/// take a file path.
#[test]
fn test_svelte_file_detection() {
    let processor = super::utils::get_processor();
    let sample_text = r#"<p>Misspeled word</p>
"#;
    let results = processor.spell_check(
        sample_text,
        Some(LanguageType::HTML),
        Some("component.svelte"),
    );
    let misspelled: Vec<&str> = results.iter().map(|r| r.word.as_str()).collect();
    assert_eq!(misspelled, ["Misspeled"]);
}
