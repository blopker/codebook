use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_swift_simple() {
    utils::init_logging();
    let processor = utils::get_processor();
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
    let expected = vec![
        "Moar",
        "Thig",
        "comented",
        "enumrable",
        "frm",
        "helo",
        "inented",
        "lne",
        "lteral",
        "nunber",
        "opttions",
        "sepaate",
        "wors",
    ];
    let binding = processor
        .spell_check(sample_text, Some(LanguageType::Swift), None)
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
fn test_swift_code() {
    utils::init_logging();
    let sample_ruby_code = r#"
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

    let expected = vec![
        WordLocation::new(
            "potentialy".to_string(),
            vec![TextRange {
                start_byte: 119,
                end_byte: 129,
            }],
        ),
        WordLocation::new(
            "compleet".to_string(),
            vec![TextRange {
                start_byte: 368,
                end_byte: 376,
            }],
        ),
        WordLocation::new(
            "notfication".to_string(),
            vec![TextRange {
                start_byte: 11,
                end_byte: 22,
            }],
        ),
        WordLocation::new(
            "Regads".to_string(),
            vec![TextRange {
                start_byte: 277,
                end_byte: 283,
            }],
        ),
        WordLocation::new(
            "complette".to_string(),
            vec![TextRange {
                start_byte: 334,
                end_byte: 343,
            }],
        ),
        WordLocation::new(
            "custommer".to_string(),
            vec![TextRange {
                start_byte: 252,
                end_byte: 261,
            }],
        ),
        WordLocation::new(
            "Suport".to_string(),
            vec![TextRange {
                start_byte: 286,
                end_byte: 292,
            }],
        ),
        WordLocation::new(
            "partialy".to_string(),
            vec![TextRange {
                start_byte: 359,
                end_byte: 367,
            }],
        ),
    ];
    let not_expected = vec!["finnished"];
    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_ruby_code, Some(LanguageType::Swift), None)
        .to_vec();
    println!("Misspelled words: {misspelled:?}");
    for e in &expected {
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        println!("Expecting: {e:?}");
        assert_eq!(miss.locations, e.locations);
    }
    for word in not_expected {
        println!("Not expecting: {word:?}");
        assert!(!misspelled.iter().any(|r| r.word == word));
    }
}
