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
                start_byte: 7,
                end_byte: 12,
            }],
        ),
        WordLocation::new(
            "Blck".to_string(),
            vec![TextRange {
                start_byte: 7,
                end_byte: 11,
            }],
        ),
        WordLocation::new(
            "Exampl".to_string(),
            vec![TextRange {
                start_byte: 14,
                end_byte: 20,
            }],
        ),
        WordLocation::new(
            "Somethng".to_string(),
            vec![
                TextRange {
                    start_byte: 15,
                    end_byte: 23,
                },
                TextRange {
                    start_byte: 22,
                    end_byte: 30,
                },
            ],
        ),
        WordLocation::new(
            "Statuss".to_string(),
            vec![TextRange {
                start_byte: 9,
                end_byte: 16,
            }],
        ),
        WordLocation::new(
            "ACTIV".to_string(),
            vec![TextRange {
                start_byte: 19,
                end_byte: 24,
            }],
        ),
        WordLocation::new(
            "Soem".to_string(),
            vec![TextRange {
                start_byte: 17,
                end_byte: 21,
            }],
        ),
        WordLocation::new(
            "messag".to_string(),
            vec![TextRange {
                start_byte: 15,
                end_byte: 21,
            }],
        ),
        WordLocation::new(
            "smth".to_string(),
            vec![TextRange {
                start_byte: 38,
                end_byte: 42,
            }],
        ),
        WordLocation::new(
            "errorr".to_string(),
            vec![TextRange {
                start_byte: 41,
                end_byte: 47,
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
