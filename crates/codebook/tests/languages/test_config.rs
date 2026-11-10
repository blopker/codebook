use codebook::{Codebook, queries::LanguageType};
use codebook_config::{CodebookConfig, CodebookConfigMemory};
use std::sync::Arc;

fn get_processor_with_words(words: &[&str]) -> Codebook {
    let config = Arc::new(CodebookConfigMemory::default());
    for w in words {
        config.add_word(w);
    }
    super::utils::make_codebook(config)
}

#[test]
fn test_custom_words() {
    let sample_text = r#"
        ok words
        testword
        good words
        actualbad
"#;
    // "testword" is added to the config dictionary, so only "actualbad" is flagged.
    super::utils::assert_spelling_with(
        &get_processor_with_words(&["testword"]),
        LanguageType::Text,
        sample_text,
        &["actualbad"],
        &["testword"],
    );
}

#[test]
fn test_override_tags_apply_per_file() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        temp_dir.path().join("codebook.toml"),
        r#"
        include_tags = ["comment", "identifier"]

        [[overrides]]
        paths = ["docs/**"]
        extra_include_tags = ["string"]

        [[overrides]]
        paths = ["notes/**"]
        include_tags = ["string"]

        [[overrides]]
        paths = ["drafts/**"]
        extra_exclude_tags = ["identifier"]

        [[overrides]]
        paths = ["legacy/**"]
        exclude_tags = ["comment"]
        "#,
    )
    .unwrap();
    let config =
        Arc::new(codebook_config::CodebookConfigFile::load(Some(temp_dir.path())).unwrap());
    let processor = super::utils::make_codebook(config);

    let sample_text = "# a coment\nvaule = \"a strng\"\n";
    let flagged = |path: &str| {
        let mut words: Vec<String> = processor
            .spell_check(sample_text, Some(LanguageType::Python), Some(path))
            .into_iter()
            .map(|r| r.word)
            .collect();
        words.sort();
        words
    };
    assert_eq!(flagged("src/example.py"), ["coment", "vaule"]);
    assert_eq!(flagged("docs/example.py"), ["coment", "strng", "vaule"]);
    assert_eq!(flagged("notes/example.py"), ["strng"]);
    assert_eq!(flagged("drafts/example.py"), ["coment"]);
    assert_eq!(flagged("legacy/example.py"), ["vaule"]);
}
