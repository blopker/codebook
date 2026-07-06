use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_just_comment() {
    assert_spelling(
        LanguageType::Just,
        "# A comentt with a tyypo\nbuild:\n    echo hi\n",
        &["comentt", "tyypo"],
        &[],
    );
}

#[test]
fn test_just_recipe_name_and_parameters() {
    // Recipe names and parameter names are checked.
    assert_spelling(
        LanguageType::Just,
        "buidl targt=\"debug\":\n    echo hi\n",
        &["buidl", "targt"],
        &[],
    );
}

#[test]
fn test_just_assignment_and_string() {
    assert_spelling(
        LanguageType::Just,
        "some_varable := \"a strng value\"\n",
        &["varable", "strng"],
        &[],
    );
}

#[test]
fn test_just_alias_definition() {
    // Only the alias name is a definition; the right side is a usage of a
    // recipe name that's already checked at its own definition.
    assert_spelling(
        LanguageType::Just,
        "alias bulid := build\nbuild:\n    echo hi\n",
        &["bulid"],
        &[],
    );
}

#[test]
fn test_just_setting_and_import_strings_skipped() {
    // Setting values and import paths are configuration, not prose.
    assert_spelling(
        LanguageType::Just,
        "set shell := [\"bsh\", \"-c\"]\nimport 'foo/badspeling.just'\n",
        &[],
        &["bsh", "badspeling"],
    );
}

#[test]
fn test_just_recipe_body_checked_as_bash() {
    let sample_text = r#"build:
    # a shel comentt
    echo "a strng with a tyypo"
    mkdir -p out
"#;
    // Comments and strings come from the injected bash grammar;
    // bash.scm doesn't capture command invocations, so mkdir isn't checked.
    assert_spelling(
        LanguageType::Just,
        sample_text,
        &["shel", "comentt", "strng", "tyypo"],
        &["mkdir"],
    );
}

#[test]
fn test_just_shebang_recipe_uses_language_grammar() {
    let sample_text = r#"build:
    #!/usr/bin/env python3
    # python comentt here
    msg = "a strng with a tyypo"
    def my_functin():
        pass
"#;
    // The shebang line itself is not spell-checked (usr, env).
    assert_spelling(
        LanguageType::Just,
        sample_text,
        &["comentt", "strng", "tyypo", "functin"],
        &["usr", "env"],
    );
}

#[test]
fn test_just_shebang_unknown_language_skipped() {
    assert_spelling(
        LanguageType::Just,
        "build:\n    #!/usr/bin/env unknownlang\n    badwwword stuff\n",
        &[],
        &["badwwword"],
    );
}

#[test]
fn test_just_interpolation_no_duplicate_spans() {
    // Strings inside interpolations are covered by the bash injection;
    // make sure the just-level string capture doesn't double-report them:
    // a duplicated span would show up as an extra, unexpected range.
    assert_spelling_at(
        LanguageType::Just,
        "build:\n    echo {{ \"a strng\" }}\n",
        &[("strng", &[0])],
    );
}

/// Ranges reported from the injected bash region must map back to original
/// document coordinates; `spell_check` in utils asserts every reported range
/// slices back to its word.
#[test]
fn test_just_injected_region_byte_offsets() {
    assert_spelling(
        LanguageType::Just,
        "build:\n    echo \"a tyypo here\"\n",
        &["tyypo"],
        &[],
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
