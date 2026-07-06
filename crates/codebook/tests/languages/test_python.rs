use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at, assert_spelling_with, get_processor};

#[test]
fn test_python_simple() {
    let sample_text = r#"
        def calculat_user_age(bithDate) -> int:
            # This is an examle_function that calculates age
            usrAge = get_curent_date() - bithDate
            userAge
    "#;
    // "bith" is flagged at the parameter definition, not at the usage in the
    // function body; call names ("curent") and assignment targets ("usr") are
    // not checked.
    assert_spelling_at(
        LanguageType::Python,
        sample_text,
        &[("bith", &[0]), ("calculat", &[0]), ("examle", &[0])],
    );
}

#[test]
fn test_python_multi_line_comment() {
    let sample_python = r#"
multi_line_comment = '''
    This is a multi line comment with a typo: mment
    Another linet
'''
        "#;
    // "mment" also appears inside "multi_line_comment" and "comment"
    // (occurrences 0 and 1); only the standalone typo in the string is
    // flagged.
    assert_spelling_at(
        LanguageType::Python,
        sample_python,
        &[("mment", &[2]), ("linet", &[0])],
    );
}

#[test]
fn test_python_class() {
    let sample_python = r#"
class BadSpelin:
    def nospel(self):
        return self.zzzznomethod() # This should not get checked
    def bad_spelin(self): # This should get checked
        return "Spelling is hardz" # This should get checked

@decorated
def constructor():
    return BadSpelin(hardx=bad.hardd, thing="hardg")  # Some of this should get checked
'''
        "#;
    // "Spelin" is flagged at the class definition only, not at the
    // constructor call; method call names ("zzzznomethod"), keyword argument
    // names ("hardx"), and attribute accesses ("hardd") are not checked.
    assert_spelling_at(
        LanguageType::Python,
        sample_python,
        &[
            ("Spelin", &[0]),
            ("nospel", &[0]),
            ("spelin", &[0]),
            ("hardz", &[0]),
            ("hardg", &[0]),
        ],
    );
}

#[test]
fn test_python_global_variables() {
    let sample_text = r#"
# Globul variables
globalCountr = 0
mesage = "Helllo Wolrd!"
    "#;
    assert_spelling(
        LanguageType::Python,
        sample_text,
        &["Globul", "Countr", "mesage", "Helllo", "Wolrd"],
        &[],
    );
}

#[test]
fn test_python_f_strings() {
    let sample_text = r#"
name = "John"
age = 25
message = f'Hello, my naem is {namz} and I am {age} years oldd'
another = f"This is antoher examle with {name} varibles"
simple = f'check these wordz {but} {not} {the} {variables}'
    "#;
    // Text content of f-strings is checked, but interpolated expressions
    // ({namz}, {age}, ...) are not.
    assert_spelling(
        LanguageType::Python,
        sample_text,
        &["naem", "oldd", "antoher", "examle", "varibles", "wordz"],
        &["namz", "age", "but", "not", "the", "variables"],
    );
}

#[test]
fn test_python_import_statements() {
    let sample_text = r#"
        import no_typpoa
        import no_typpob.no_typpoc

        import no_typpod as yes_typpoe
        import no_typpof.no_typpog as yes_typpoh

        from no_typpoi import no_typpoj
        from no_typpok.no_typpol import no_typpom

        from no_typpoo import no_typpop as yes_typpoq
        from no_typpor.no_typpos import no_typpot as yes_typpou
        from .. import no_typpov as yes_typpow
    "#;
    // Only local aliases (`as yes_*`) are checked; imported module and
    // symbol names (`no_*`) are not.
    assert_spelling(
        LanguageType::Python,
        sample_text,
        &["typpoe", "typpoh", "typpoq", "typpou", "typpow"],
        &["typpoa", "typpoc", "typpoj", "typpom", "typpov"],
    );
}

#[test]
fn test_python_type_annotations() {
    // Variable annotations, parameter annotations, and return types — both
    // bare identifiers and string forward references — should be ignored.
    // Regression test for https://github.com/blopker/codebook/issues/187.
    let sample = r#"
from typing import Union

a: no_typpoa = ...
b: 'no_typpob' = ...
c: "no_typpoc" = ...
d: """no_typpod""" = ...
e: str | no_typpoe | "no_typpof" = ...
f: Union[str, no_typpog, "no_typpoh"] = ...
g: list["no_typpoi"] = ...

def func(
    param_a: no_typpoj,
    param_b: 'no_typpok',
    param_d: no_typpom = ...,
):
    pass

def func2() -> str | no_typpon | "no_typpoo":
    pass
    "#;
    assert_spelling(
        LanguageType::Python,
        sample,
        &[],
        &[
            "typpoa", "typpob", "typpoc", "typpod", "typpoe", "typpof", "typpog", "typpoh",
            "typpoi", "typpoj", "typpok", "typpom", "typpon", "typpoo",
        ],
    );
}

#[test]
fn test_python_functions() {
    let processor = get_processor();

    // Simple function - function name and parameter names should be checked
    let simple_function = r#"
def simple_wrngfunction_name(wrngparam, correct, wrngdefaultparam=1, correct_default=2):
    pass
    "#;
    assert_spelling_with(
        &processor,
        LanguageType::Python,
        simple_function,
        &["wrngfunction", "wrngparam", "wrngdefaultparam"],
        &["simple", "correct", "def", "name", "default"],
    );

    // Typed function - function names and parameters should be checked, but
    // not types or modules
    let simple_typed_function = r#"
def simple_wrngfunction(wrngparam: str, correct: Wrngtype, other: wrngmod.Wrngmodtype, correct_default: Nons | int = 2) -> Wrngret:
    pass
    "#;
    assert_spelling_with(
        &processor,
        LanguageType::Python,
        simple_typed_function,
        &["wrngfunction", "wrngparam"],
        &[
            "simple",
            "correct",
            "str",
            "Wrngtype",
            "wrngmod",
            "Wrngmodtype",
            "Wrngret",
            "def",
            "Nons",
            "default",
        ],
    );

    // Generic function 1 - function names and parameters should be checked,
    // but not types
    let generic_function_1 = r#"
def simple_wrngfunction(wrngparam: str, correct: Wrngtype[Wrngtemplate]):
    pass
    "#;
    assert_spelling_with(
        &processor,
        LanguageType::Python,
        generic_function_1,
        &["wrngfunction", "wrngparam"],
        &["simple", "correct", "str", "Wrngtype", "Wrngtemplate"],
    );

    // Generic function 2 - function names and parameters should be checked,
    // but not type templates
    let generic_function_2 = r#"
def simple_wrngfunction[Wrgtemplate](wrngparam: str, correct: Wrngtype[Wrngtemplate]):
    pass
    "#;
    assert_spelling_with(
        &processor,
        LanguageType::Python,
        generic_function_2,
        &["wrngfunction", "wrngparam"],
        &[
            "simple",
            "correct",
            "str",
            "Wrgtemplate",
            "Wrngtype",
            "Wrngtemplate",
        ],
    );
}
