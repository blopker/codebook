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
                start_char: 7,
                end_char: 12,
                line: 1,
            }],
        ),
        WordLocation::new(
            "Blck".to_string(),
            vec![TextRange {
                start_char: 7,
                end_char: 11,
                line: 2,
            }],
        ),
        WordLocation::new(
            "Exampl".to_string(),
            vec![TextRange {
                start_char: 14,
                end_char: 20,
                line: 4,
            }],
        ),
        WordLocation::new(
            "Somethng".to_string(),
            vec![
                TextRange {
                    start_char: 15,
                    end_char: 23,
                    line: 5,
                },
                TextRange {
                    start_char: 22,
                    end_char: 30,
                    line: 14,
                },
            ],
        ),
        WordLocation::new(
            "Statuss".to_string(),
            vec![TextRange {
                start_char: 9,
                end_char: 16,
                line: 8,
            }],
        ),
        WordLocation::new(
            "ACTIV".to_string(),
            vec![TextRange {
                start_char: 19,
                end_char: 24,
                line: 8,
            }],
        ),
        WordLocation::new(
            "Soem".to_string(),
            vec![TextRange {
                start_char: 17,
                end_char: 21,
                line: 10,
            }],
        ),
        WordLocation::new(
            "messag".to_string(),
            vec![TextRange {
                start_char: 15,
                end_char: 21,
                line: 12,
            }],
        ),
        WordLocation::new(
            "smth".to_string(),
            vec![TextRange {
                start_char: 38,
                end_char: 42,
                line: 14,
            }],
        ),
        WordLocation::new(
            "errorr".to_string(),
            vec![TextRange {
                start_char: 41,
                end_char: 47,
                line: 21,
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
