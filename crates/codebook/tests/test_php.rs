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
                start_byte: 49,
                end_byte: 57,
            }],
        ),
        WordLocation::new(
            "Servicce".to_string(),
            vec![TextRange {
                start_byte: 112,
                end_byte: 120,
            }],
        ),
        WordLocation::new(
            "ACTIVVE".to_string(),
            vec![TextRange {
                start_byte: 163,
                end_byte: 170,
            }],
        ),
        WordLocation::new(
            "Idd".to_string(),
            vec![
                TextRange {
                    start_byte: 219,
                    end_byte: 222,
                },
                TextRange {
                    start_byte: 307,
                    end_byte: 310,
                },
            ],
        ),
        WordLocation::new(
            "databaase".to_string(),
            vec![
                TextRange {
                    start_byte: 239,
                    end_byte: 248,
                },
                TextRange {
                    start_byte: 313,
                    end_byte: 322,
                },
            ],
        ),
        WordLocation::new(
            "Deetails".to_string(),
            vec![TextRange {
                start_byte: 473,
                end_byte: 481,
            }],
        ),
        WordLocation::new(
            "querry".to_string(),
            vec![TextRange {
                start_byte: 495,
                end_byte: 501,
            }],
        ),
        WordLocation::new(
            "foundd".to_string(),
            vec![TextRange {
                start_byte: 630,
                end_byte: 636,
            }],
        ),
        WordLocation::new(
            "formatt".to_string(),
            vec![TextRange {
                start_byte: 722,
                end_byte: 729,
            }],
        ),
        WordLocation::new(
            "amountt".to_string(),
            vec![TextRange {
                start_byte: 739,
                end_byte: 746,
            }],
        ),
        WordLocation::new(
            "symboll".to_string(),
            vec![TextRange {
                start_byte: 774,
                end_byte: 781,
            }],
        ),
        WordLocation::new(
            "formattted".to_string(),
            vec![TextRange {
                start_byte: 855,
                end_byte: 865,
            }],
        ),
        WordLocation::new(
            "errr".to_string(),
            vec![TextRange {
                start_byte: 930,
                end_byte: 934,
            }],
        ),
        WordLocation::new(
            "userr".to_string(),
            vec![TextRange {
                start_byte: 1019,
                end_byte: 1024,
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
        assert!(miss.locations.len() == e.locations.len());
        for location in &miss.locations {
            assert!(e.locations.contains(location));
        }
    }

    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
