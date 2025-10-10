use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;
// WIP PHP Support
#[test]
fn test_php_location() {
    utils::init_logging();
    let sample_text = r#"<?php
// This is a PHP sample file
namespace App\Servicez;

/**
 * A class with some misspellings
 */
class UserServicce {
    // Class constants
    const STATUS_ACTIVVE = 'active';

    // Properties
    private $userIdd;
    protected $databaase;

    // Constructor
    public function __construct($userIdd, $databaase) {
        $this->userIdd = $userIdd;
        $this->databaase = $databaase;
    }

    // Regular method with misspelling
    public function getUserDeetails() {
        $querry = "SELECT * FROM users WHERE id = " . $this->userIdd;

        if (empty($resullt)) {
            throw new \Excepton("User not foundd");
        }

        return $resullt;
    }
}

// Function outside a class
function formattCurrency($amountt, $currency = 'USD') {
    $symboll = '';

    try {
        // Some code that might throw an error
        $formattted = $symboll . number_format($amountt, 2);
    } catch (Excepton $errr) {
        // Handle the error
    }

    return $formattted;
}

// Variable usage
$userr = new UserServicce(123, $dbb);
$userDetails = $userr->getUserDeetails();
?>"#;

    let expected = vec![
        WordLocation::new(
            "Servicez".to_string(),
            vec![TextRange {
                start_byte: 14,
                end_byte: 22,
            }],
        ),
        WordLocation::new(
            "Servicce".to_string(),
            vec![TextRange {
                start_byte: 10,
                end_byte: 18,
            }],
        ),
        WordLocation::new(
            "ACTIVVE".to_string(),
            vec![TextRange {
                start_byte: 17,
                end_byte: 24,
            }],
        ),
        WordLocation::new(
            "Idd".to_string(),
            vec![
                TextRange {
                    start_byte: 17,
                    end_byte: 20,
                },
                TextRange {
                    start_byte: 37,
                    end_byte: 40,
                },
            ],
        ),
        WordLocation::new(
            "databaase".to_string(),
            vec![
                TextRange {
                    start_byte: 15,
                    end_byte: 24,
                },
                TextRange {
                    start_byte: 43,
                    end_byte: 52,
                },
            ],
        ),
        WordLocation::new(
            "Deetails".to_string(),
            vec![TextRange {
                start_byte: 27,
                end_byte: 35,
            }],
        ),
        WordLocation::new(
            "querry".to_string(),
            vec![TextRange {
                start_byte: 9,
                end_byte: 15,
            }],
        ),
        WordLocation::new(
            "foundd".to_string(),
            vec![TextRange {
                start_byte: 42,
                end_byte: 48,
            }],
        ),
        WordLocation::new(
            "formatt".to_string(),
            vec![TextRange {
                start_byte: 9,
                end_byte: 16,
            }],
        ),
        WordLocation::new(
            "amountt".to_string(),
            vec![TextRange {
                start_byte: 26,
                end_byte: 33,
            }],
        ),
        WordLocation::new(
            "symboll".to_string(),
            vec![TextRange {
                start_byte: 5,
                end_byte: 12,
            }],
        ),
        WordLocation::new(
            "formattted".to_string(),
            vec![TextRange {
                start_byte: 9,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "errr".to_string(),
            vec![TextRange {
                start_byte: 23,
                end_byte: 27,
            }],
        ),
        WordLocation::new(
            "userr".to_string(),
            vec![TextRange {
                start_byte: 1,
                end_byte: 6,
            }],
        ),
    ];

    let not_expected = [
        "Excepton",
        "php",
        "namespace",
        "class",
        "function",
        "private",
        "protected",
        "public",
        "const",
        "try",
        "catch",
        "new",
        "return",
        "throw",
        "empty",
    ];

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Php), None)
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
