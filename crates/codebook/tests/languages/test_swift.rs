use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_swift_simple() {
    let sample_text = r#"
        // Misspell on a sepaate line
        class Object { // comment at the end of the lne
            // Comment can be inented
            func bar() {
            }
            func opttions() {
            }
        }

        /* func foobar()
         * {
         * These wors are
         * comented out but should be identified
         */

        func doStuff(_ nunber: Int)
        {
        }
        func doMoar(_ nunber: Int)
        {
        }
        func doAgain(frm: number: Int)
        {
        }
        class Foo2 {
        class MyThig {
        }

        // frozen_string_lteral: true

        var x = "helo"

        protocol enumrable {
        }
    "#;
    assert_spelling_at(
        LanguageType::Swift,
        sample_text,
        &[
            ("sepaate", &[0]),
            ("lne", &[0]),
            ("inented", &[0]),
            ("opttions", &[0]),
            // Block-comment contents are checked.
            ("wors", &[0]),
            ("comented", &[0]),
            // Both parameter definitions (doStuff and doMoar) are flagged.
            ("nunber", &[0, 1]),
            ("Moar", &[0]),
            ("frm", &[0]),
            ("Thig", &[0]),
            ("lteral", &[0]),
            ("helo", &[0]),
            ("enumrable", &[0]),
        ],
    );
}

#[test]
fn test_swift_code() {
    let sample_text = r#"
func send_notfication(to recipient: String, _ subject: String, body: String)
{
    // This method sends an email with potentialy misspelled content
    let email = Email(to: recipient,
        subject: "URGENT: #{subject}",
        body: "Dear valued custommer,\n\n#{body}\n\nRegads,\nSuport Team")
    email.send()
}

if status == "complette" || status == "partialy_compleet" {
    mark_as_finnished(item)
}
        "#;
    assert_spelling(
        LanguageType::Swift,
        sample_text,
        &[
            "notfication",
            "potentialy",
            "custommer",
            "Regads",
            "Suport",
            "complette",
            // Split from the string "partialy_compleet".
            "partialy",
            "compleet",
        ],
        // Function-call references are not spell-checked.
        &["finnished"],
    );
}
