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
