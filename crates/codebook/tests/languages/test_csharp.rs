use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_csharp_simple() {
    let sample_text = r#"
class Demo {
    const int tesst = 5;

    static void Run() {
        var valuue = 10;
    }
}
"#;
    assert_spelling(
        LanguageType::CSharp,
        sample_text,
        &["tesst", "valuue"],
        &["Demo", "Run"],
    );
}

#[test]
fn test_csharp_strings_and_comments() {
    let sample_text = r#"
// Comment with speling error.

/* Multi-line comment
with error on seconnd line */

class DemoStrings {
    static void Test() {
        var msg = "Hello Wolrd"; // inline coment with eror
        System.Console.WriteLine(msg);

        var interpolated = $"Helo {msg}";
        System.Console.WriteLine(interpolated);

        var multi = @"ernor
           spelingg
        ");
        System.Console.WriteLine(multi);
    }
}
"#;
    assert_spelling_at(
        LanguageType::CSharp,
        sample_text,
        &[
            // Occurrence 1 is the substring inside `spelingg`, which is
            // flagged as its own whole word instead.
            ("speling", &[0]),
            ("seconnd", &[0]),
            ("Wolrd", &[0]),
            ("coment", &[0]),
            ("eror", &[0]),
            ("Helo", &[0]),
            ("ernor", &[0]),
            ("spelingg", &[0]),
        ],
    );
}

#[test]
fn test_csharp_functions() {
    let sample_text = r#"
class MathUtil {
    static int AddNumberrs(int firstt, int seconnd) {
        int resullt = firstt + seconnd;
        return resullt;
    }
}
"#;
    assert_spelling_at(
        LanguageType::CSharp,
        sample_text,
        &[
            // Flagged at the `Numberrs` sub-token range inside `AddNumberrs`.
            ("Numberrs", &[0]),
            // Parameters and locals are flagged at their declaration, not at
            // the expression or return usages.
            ("firstt", &[0]),
            ("seconnd", &[0]),
            ("resullt", &[0]),
        ],
    );
}
