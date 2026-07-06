use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

/// Anchor test for the tree-sitter (code) path: strict positional checking
/// via occurrence indices. The emoji in the comment and string sit BEFORE
/// most misspellings so multi-byte offset drift shifts the reported ranges
/// and fails the comparison. Occurrence indices count case-sensitive
/// substring matches: e.g. `Userr` occurs inside `GetUserr` because
/// camelCase splitting flags it at that sub-token range.
#[test]
fn test_go_location() {
    let sample_text = r#"
    package pacagename
    import myfmt "fmt"
    type Userr struct {
        Namee string `json:"namme"`
    }
    type UserServicce interface {
        GetUserr(id string) Userr
    }
    const MaxNameeSize = 100
    func (u *Userr) GetUserr(prefixx string) Userr {
        return Userr{Namee: prefixx + "Alice"}
    }
    func main() {
        // I'm bad at speling 😀 alice
        myfmt.Println("Hello, 🌍 Wolrd!")
        var alicz = "Alicz"
        myfmt.Println("Hellol, " + alicz)
        var rsvp = "RSVP"
        myfmt.Println("Hello, " + rsvp)
        cokbookkk := "test valie"
        myfmt.Println("Hello, " + cokbookkk)
    outerr:
        for imdex := 0; imdex < 10; imdex++ {
            if imdex == 5 {
                break outerr
            }
        }
        itemns := []string{"firstt", "seconnd", "tihrd"}
        for indexx, valuue := range itemns {
            myfmt.Println(indexx, valuue)
        }
        myfmt.Println(itemns)
    }"#;
    assert_spelling_at(
        LanguageType::Go,
        sample_text,
        &[
            ("pacagename", &[0]),
            // Identifiers are flagged at their definition, not at usages.
            ("myfmt", &[0]),
            // All 7 occurrences: type name, inside both GetUserr methods
            // (camelCase split), receiver, return types, and struct literal.
            ("Userr", &[0, 1, 2, 3, 4, 5, 6]),
            ("prefixx", &[0]),
            // Struct field definition and inside MaxNameeSize (camelCase
            // split); the `Userr{Namee:` usage is not flagged.
            ("Namee", &[0, 1]),
            ("Servicce", &[0]),
            ("namme", &[0]),
            // Labels are flagged at definition and at `break`.
            ("outerr", &[0, 1]),
            ("itemns", &[0]),
            ("indexx", &[0]),
            ("cokbookkk", &[0]),
            ("valie", &[0]),
            ("Wolrd", &[0]),
            ("Alicz", &[0]),
            ("alicz", &[0]),
            ("speling", &[0]),
            ("Hellol", &[0]),
            ("imdex", &[0]),
            ("firstt", &[0]),
            ("seconnd", &[0]),
            ("tihrd", &[0]),
            ("valuue", &[0]),
        ],
    );
}

#[test]
fn test_go_imports_not_checked() {
    let sample_text = r#"
    package main

    import (
        "fmt"
        "net/http"
        "github.com/someuserr/mypackagee"
        myfmt "github.com/anotherr/fmtpkg"
    )

    func main() {
        fmt.Println("hello")
    }"#;
    assert_spelling(
        LanguageType::Go,
        sample_text,
        // Import aliases are still checked.
        &["myfmt"],
        // Import path contents are not spell-checked.
        &["someuserr", "mypackagee", "anotherr", "fmtpkg"],
    );
}
