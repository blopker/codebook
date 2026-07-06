use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at_with, assert_spelling_with};

#[test]
fn test_text_with_urls_should_skip_misspelled_words_in_urls() {
    // URLs contain "misspelled" words like "exampl", "badspeling" that should be ignored
    let sample_text = r#"
        Visit https://www.exampl.com/badspeling for more info.
        Also check out http://github.com/usr/repositry/issues
        But this actualbadword should be flagged.
    "#;
    assert_spelling(
        LanguageType::Text,
        sample_text,
        &["actualbadword"],
        &["exampl", "badspeling", "repositry"],
    );
}

#[test]
fn test_text_with_hex_colors_should_skip() {
    // Hex colors that might contain letter patterns that look like words
    let sample_text = r#"
        Set the color to #deadbeef for the background.
        Use #bada55 or #facade for highlights.
        But this badcolorname should be flagged.
    "#;
    assert_spelling(
        LanguageType::Text,
        sample_text,
        &["badcolorname"],
        &["deadbeef", "bada", "facade"],
    );
}

#[test]
fn test_text_with_emails_should_skip() {
    let sample_text = r#"
        Contact usr@exampl.com or admin@badspeling.org
        This misspelledword should be flagged though.
    "#;
    assert_spelling(
        LanguageType::Text,
        sample_text,
        &["misspelledword"],
        &["exampl", "badspeling"],
    );
}

#[test]
fn test_python_with_urls_in_strings_should_skip() {
    let sample_text = r#"
        def fetch_data():
            # Visit https://api.exampl.com/badspeling/endpoint
            url = "https://github.com/usr/badrepo"
            return requests.get(url)

        def badmethodname():  # This should be flagged
            pass
    "#;
    assert_spelling(
        LanguageType::Python,
        sample_text,
        &["badmethodname"],
        // URL parts should not be flagged
        &["exampl", "badspeling", "badrepo"],
    );
}

#[test]
fn test_python_with_hex_colors_should_skip() {
    let sample_text = r##"
        def set_colors():
            primary_color = "#deadbeef"
            secondary = "#bada55"
            highlight = "#facade"

        def badcolormethod():  # This should be flagged
            return "#000000"
            "##;
    assert_spelling(
        LanguageType::Python,
        sample_text,
        &["badcolormethod"],
        &["deadbeef", "bada", "facade"],
    );
}

#[test]
fn test_multiple_patterns_combined() {
    let sample_text = r#"
        Visit https://exampl.com/badspeling
        Email: usr@baddomaine.com
        Color: #deadbeef
        Path: /usr/badpath/file.txt
        This actualbadword should be flagged.
    "#;
    assert_spelling(
        LanguageType::Text,
        sample_text,
        &["actualbadword"],
        &["exampl", "badspeling", "baddomaine", "deadbeef", "badpath"],
    );
}

#[test]
fn test_user_defined_regex_patterns() {
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

    let processor = super::utils::make_codebook(config);

    let sample_text = r#"
        This text has HTML and CSS frameworks.
        Also customword and testpattern should be ignored.
        The date 2024-01-15 should be skipped too.
        But badword and anotherbadword should be flagged.
    "#;
    // Exact set equality: HTML, CSS, customword, testpattern, and the date are
    // all skipped by the user patterns. "badword" is a substring of
    // "anotherbadword", hence occurrence indices: only the standalone
    // occurrence (index 0) is flagged as "badword".
    assert_spelling_at_with(
        &processor,
        LanguageType::Text,
        sample_text,
        &[("badword", &[0]), ("anotherbadword", &[0])],
    );
}

#[test]
fn test_pattern_matching_against_full_source() {
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

    let processor = super::utils::make_codebook(config);

    // Lua code with vim.opt settings. "showmode" and "relativenumber" fall
    // within the matched ranges of "vim.opt.showmode" and
    // "vim.opt.relativenumber", so they are skipped.
    let sample_text = r#"
        vim.opt.showmode = false
        vim.opt.relativenumber = true
        local badword = "test"
    "#;
    assert_spelling_with(
        &processor,
        LanguageType::Lua,
        sample_text,
        &["badword"],
        &["showmode", "relativenumber"],
    );
}
