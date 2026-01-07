use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_text_with_urls_should_skip_misspelled_words_in_urls() {
    utils::init_logging();
    let processor = utils::get_processor();

    // URLs contain "misspelled" words like "exampl", "badspeling" that should be ignored
    let sample_text = r#"
        Visit https://www.exampl.com/badspeling for more info.
        Also check out http://github.com/usr/repositry/issues
        But this actualbadword should be flagged.
    "#;

    // Only "actualbadword" should be flagged, not "exampl", "badspeling", "repositry"
    let expected = vec!["actualbadword"];

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);

    // Verify URLs words are NOT in the results
    assert!(!misspelled.contains(&"exampl"));
    assert!(!misspelled.contains(&"badspeling"));
    assert!(!misspelled.contains(&"repositry"));
}

#[test]
fn test_text_with_hex_colors_should_skip() {
    utils::init_logging();
    let processor = utils::get_processor();

    // Hex colors that might contain letter patterns that look like words
    let sample_text = r#"
        Set the color to #deadbeef for the background.
        Use #bada55 or #facade for highlights.
        But this badcolorname should be flagged.
    "#;

    // Only "badcolorname" should be flagged
    let expected = vec!["badcolorname"];

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);

    // Verify hex color parts are NOT flagged
    assert!(!misspelled.contains(&"deadbeef"));
    assert!(!misspelled.contains(&"bada"));
    assert!(!misspelled.contains(&"facade"));
}

#[test]
fn test_text_with_emails_should_skip() {
    utils::init_logging();
    let processor = utils::get_processor();

    let sample_text = r#"
        Contact usr@exampl.com or admin@badspeling.org
        This misspelledword should be flagged though.
    "#;

    let expected = vec!["misspelledword"];

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_python_with_urls_in_strings_should_skip() {
    utils::init_logging();
    let processor = utils::get_processor();

    let sample_text = r#"
        def fetch_data():
            # Visit https://api.exampl.com/badspeling/endpoint
            url = "https://github.com/usr/badrepo"
            return requests.get(url)

        def badmethodname():  # This should be flagged
            pass
    "#;

    let expected = vec!["badmethodname"];

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Python), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);

    // URL parts should not be flagged
    assert!(!misspelled.contains(&"exampl"));
    assert!(!misspelled.contains(&"badspeling"));
    assert!(!misspelled.contains(&"badrepo"));
}

#[test]
fn test_python_with_hex_colors_should_skip() {
    utils::init_logging();
    let processor = utils::get_processor();

    let sample_text = r##"
        def set_colors():
            primary_color = "#deadbeef"
            secondary = "#bada55"
            highlight = "#facade"

        def badcolormethod():  # This should be flagged
            return "#000000"
            "##;

    let expected = vec!["badcolormethod"];

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Python), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_multiple_patterns_combined() {
    utils::init_logging();
    let processor = utils::get_processor();

    let sample_text = r#"
        Visit https://exampl.com/badspeling
        Email: usr@baddomaine.com
        Color: #deadbeef
        Path: /usr/badpath/file.txt
        This actualbadword should be flagged.
    "#;

    let expected = vec!["actualbadword"];

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");
    assert_eq!(misspelled, expected);
}

#[test]
fn test_user_defined_regex_patterns() {
    utils::init_logging();

    // Create a temporary config with user-defined patterns
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codebook.toml");

    // Test multiple types of user-defined regex patterns
    let config_content = r#"
        ignore_patterns = [
            "^[A-Z]{2,}$",           # All caps words like "HTML", "CSS"
            "\\bcustom\\w*",             # Words starting with "custom"
            "\\d{4}-\\d{2}-\\d{2}",  # Date format like "2024-01-15"
            "testpattern"            # Simple literal match
        ]
    "#;

    std::fs::write(&config_path, config_content).unwrap();

    let config = std::sync::Arc::new(
        codebook_config::CodebookConfigFile::load(Some(temp_dir.path())).unwrap(),
    );

    let processor = codebook::Codebook::new(config).unwrap();

    let sample_text = r#"
        This text has HTML and CSS frameworks.
        Also customword and testpattern should be ignored.
        The date 2024-01-15 should be skipped too.
        But badword and anotherbadword should be flagged.
    "#;

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Text), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    // Verify words that should be skipped by user patterns
    assert!(
        !misspelled.contains(&"HTML"),
        "HTML should be skipped by ^[A-Z]{{2,}}$ pattern"
    );
    assert!(
        !misspelled.contains(&"CSS"),
        "CSS should be skipped by ^[A-Z]{{2,}}$ pattern"
    );
    assert!(
        !misspelled.contains(&"customword"),
        "customword should be skipped by ^custom.* pattern"
    );
    assert!(
        !misspelled.contains(&"testpattern"),
        "testpattern should be skipped by literal pattern"
    );

    // Verify words that should still be flagged (not matching any user pattern)
    assert!(
        misspelled.contains(&"badword"),
        "badword should be flagged as it doesn't match any pattern"
    );
    assert!(
        misspelled.contains(&"anotherbadword"),
        "anotherbadword should be flagged as it doesn't match any pattern"
    );
}

#[test]
fn test_pattern_matching_against_full_source() {
    utils::init_logging();

    // Create a temporary config with a pattern that matches vim.opt.* expressions
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_path = temp_dir.path().join("codebook.toml");

    // Pattern to match "vim.opt.<identifier>" - must include the identifier to skip it
    let config_content = r#"
        ignore_patterns = [
            "vim\\.opt\\.[a-z]+"
        ]
    "#;

    std::fs::write(&config_path, config_content).unwrap();

    let config = std::sync::Arc::new(
        codebook_config::CodebookConfigFile::load(Some(temp_dir.path())).unwrap(),
    );

    let processor = codebook::Codebook::new(config).unwrap();

    // Lua code with vim.opt settings
    let sample_text = r#"
        vim.opt.showmode = false
        vim.opt.relativenumber = true
        local badword = "test"
    "#;

    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Lua), None)
        .to_vec();
    let mut misspelled = binding
        .iter()
        .map(|r| r.word.as_str())
        .collect::<Vec<&str>>();
    misspelled.sort();
    println!("Misspelled words: {misspelled:?}");

    // "showmode" and "relativenumber" should be skipped because they fall within
    // the matched range of "vim.opt.showmode" and "vim.opt.relativenumber"
    assert!(
        !misspelled.contains(&"showmode"),
        "showmode should be skipped - it's within the vim.opt.showmode match"
    );
    assert!(
        !misspelled.contains(&"relativenumber"),
        "relativenumber should be skipped - it's within the vim.opt.relativenumber match"
    );

    // "badword" should still be flagged - it's not within any skip range
    assert!(
        misspelled.contains(&"badword"),
        "badword should be flagged - it doesn't match the pattern"
    );
}
