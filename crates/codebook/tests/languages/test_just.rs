use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

#[test]
fn test_just_comment() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "# A comentt with a tyypo\nbuild:\n    echo hi\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"comentt"));
    assert!(words.contains(&"tyypo"));
}

#[test]
fn test_just_recipe_name_and_parameters() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "buidl targt=\"debug\":\n    echo hi\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"buidl"), "recipe name should be checked");
    assert!(words.contains(&"targt"), "parameter name should be checked");
}

#[test]
fn test_just_assignment_and_string() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "some_varable := \"a strng value\"\n";
    let expected = [
        WordLocation::new(
            "varable".to_string(),
            vec![TextRange {
                start_byte: 5,
                end_byte: 12,
            }],
        ),
        WordLocation::new(
            "strng".to_string(),
            vec![TextRange {
                start_byte: 19,
                end_byte: 24,
            }],
        ),
    ];
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for expected in &expected {
        let found = misspelled.iter().find(|r| r.word == expected.word);
        assert!(
            found.is_some(),
            "Expected '{}' to be flagged",
            expected.word
        );
        assert_eq!(found.unwrap().locations, expected.locations);
    }
}

#[test]
fn test_just_alias_definition() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Only the alias name is a definition; the right side is a usage of a
    // recipe name that's already checked at its own definition.
    let sample_text = "alias bulid := build\nbuild:\n    echo hi\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"bulid"));
}

#[test]
fn test_just_setting_and_import_strings_skipped() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Setting values and import paths are configuration, not prose.
    let sample_text = "set shell := [\"bsh\", \"-c\"]\nimport 'foo/badspeling.just'\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(!words.contains(&"bsh"));
    assert!(!words.contains(&"badspeling"));
}

#[test]
fn test_just_recipe_body_checked_as_bash() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"build:
    # a shel comentt
    echo "a strng with a tyypo"
    mkdir -p out
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    // Comments and strings come from the injected bash grammar
    assert!(words.contains(&"shel"));
    assert!(words.contains(&"comentt"));
    assert!(words.contains(&"strng"));
    assert!(words.contains(&"tyypo"));
    // bash.scm doesn't capture command invocations
    assert!(!words.contains(&"mkdir"));
}

#[test]
fn test_just_shebang_recipe_uses_language_grammar() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = r#"build:
    #!/usr/bin/env python3
    # python comentt here
    msg = "a strng with a tyypo"
    def my_functin():
        pass
"#;
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(words.contains(&"comentt"));
    assert!(words.contains(&"strng"));
    assert!(words.contains(&"tyypo"));
    assert!(words.contains(&"functin"));
    // The shebang line itself is not spell-checked
    assert!(!words.contains(&"usr"));
    assert!(!words.contains(&"env"));
}

#[test]
fn test_just_shebang_unknown_language_skipped() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "build:\n    #!/usr/bin/env unknownlang\n    badwwword stuff\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    let words: Vec<&str> = misspelled.iter().map(|r| r.word.as_str()).collect();
    println!("Misspelled words: {words:?}");
    assert!(!words.contains(&"badwwword"));
}

#[test]
fn test_just_interpolation_no_duplicate_spans() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    // Strings inside interpolations are covered by the bash injection;
    // make sure the just-level string capture doesn't double-report them.
    let sample_text = "build:\n    echo {{ \"a strng\" }}\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for result in &misspelled {
        let deduped: std::collections::HashSet<_> = result.locations.iter().collect();
        assert_eq!(
            result.locations.len(),
            deduped.len(),
            "Word '{}' has duplicate spans: {:?}",
            result.word,
            result.locations
        );
    }
}

#[test]
fn test_just_injected_region_byte_offsets() {
    super::utils::init_logging();
    let processor = super::utils::get_processor();
    let sample_text = "build:\n    echo \"a tyypo here\"\n";
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Just), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    let tyypo = misspelled.iter().find(|w| w.word == "tyypo");
    assert!(tyypo.is_some(), "Expected 'tyypo' to be flagged");
    let loc = &tyypo.unwrap().locations[0];
    assert_eq!(
        &sample_text[loc.start_byte..loc.end_byte],
        "tyypo",
        "Byte offsets should map back to 'tyypo' in the original document"
    );
}

#[test]
fn test_just_filename_detection() {
    use codebook::queries::get_language_name_from_filename;
    assert_eq!(
        get_language_name_from_filename("justfile"),
        LanguageType::Just
    );
    assert_eq!(
        get_language_name_from_filename("Justfile"),
        LanguageType::Just
    );
    assert_eq!(
        get_language_name_from_filename("/some/path/justfile"),
        LanguageType::Just
    );
    assert_eq!(
        get_language_name_from_filename("/some/path/.justfile"),
        LanguageType::Just
    );
    assert_eq!(
        get_language_name_from_filename("recipes.just"),
        LanguageType::Just
    );
}
