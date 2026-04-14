use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_svelte_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
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
    let expected = vec![
        // comment in script
        "componet",
        "speling",
        "erors",
        // script identifiers
        "couter",
        "incremnt",
        // html text
        "Welcom",
        "aplicaton",
        "curent",
        "Incrementt",
        // css identifier
        "containr",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::HTML), None)
        .to_vec();
    let mut misspelled: Vec<&str> = binding.iter().map(|r| r.word.as_str()).collect();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    for word in &expected {
        assert!(
            misspelled.contains(word),
            "Expected misspelled word not found: {word}"
        );
    }
}

#[test]
fn test_svelte_file_detection() {
    utils::init_logging();
    let processor = utils::get_processor();
    let sample_text = r#"<p>Misspeled word</p>
"#;
    let results = processor
        .spell_check(
            sample_text,
            Some(LanguageType::HTML),
            Some("component.svelte"),
        )
        .to_vec();
    let misspelled: Vec<&str> = results.iter().map(|r| r.word.as_str()).collect();
    assert!(misspelled.contains(&"Misspeled"));
}
