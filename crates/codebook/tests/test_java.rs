use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_java_location() {
    utils::init_logging();
    let sample_text = r#"
    // Singl-line comment
    /* Blck comment */

    interface ExamplInterface {
        void doSomethng();
    }

    enum Statuss { ACTIV }

    public class SoemJavaDemo implements ExamplInterface {

        String messag = "Hello";

        public void doSomethng(String smth) {
            System.out.println("Doing " + smth + "...");
        }

        public static void main(String[] args) {
            try {
                int x = 1 / 0;
            } catch (ArithmeticException errorr) {
                System.out.println("Caught: " + errorr);
                some.recoveryMthod();
            }
        }
    }"#;

    let expected = vec![
        WordLocation::new(
            "Singl".to_string(),
            vec![TextRange {
                start_byte: 8,
                end_byte: 13,
            }],
        ),
        WordLocation::new(
            "Blck".to_string(),
            vec![TextRange {
                start_byte: 34,
                end_byte: 38,
            }],
        ),
        WordLocation::new(
            "Exampl".to_string(),
            vec![TextRange {
                start_byte: 65,
                end_byte: 71,
            }],
        ),
        WordLocation::new(
            "Somethng".to_string(),
            vec![
                TextRange {
                    start_byte: 98,
                    end_byte: 106,
                },
                TextRange {
                    start_byte: 261,
                    end_byte: 269,
                },
            ],
        ),
        WordLocation::new(
            "Statuss".to_string(),
            vec![TextRange {
                start_byte: 126,
                end_byte: 133,
            }],
        ),
        WordLocation::new(
            "ACTIV".to_string(),
            vec![TextRange {
                start_byte: 136,
                end_byte: 141,
            }],
        ),
        WordLocation::new(
            "Soem".to_string(),
            vec![TextRange {
                start_byte: 162,
                end_byte: 166,
            }],
        ),
        WordLocation::new(
            "messag".to_string(),
            vec![TextRange {
                start_byte: 220,
                end_byte: 226,
            }],
        ),
        WordLocation::new(
            "smth".to_string(),
            vec![TextRange {
                start_byte: 277,
                end_byte: 281,
            }],
        ),
        WordLocation::new(
            "errorr".to_string(),
            vec![TextRange {
                start_byte: 492,
                end_byte: 498,
            }],
        ),
    ];

    let not_expected = [
        "interface",
        "void",
        "enum",
        "public",
        "class",
        "implements",
        "String",
        "System",
        "out",
        "println",
        "static",
        "main",
        "try",
        "catch",
        "ArithmeticException",
        "Hello",
        "Doing",
        "Caught",
        "Mthod",
    ];

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Java), None)
        .to_vec();

    println!("Misspelled words: {misspelled:?}\n");

    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled
            .iter()
            .find(|r| r.word == e.word)
            .expect("Word not found");
        assert_eq!(miss.locations, e.locations);
    }

    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
