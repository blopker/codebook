use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

// WIP PHP Support
#[test]
fn test_php_location() {
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

    // Identifiers are flagged at their declaration, not at usages: class
    // names ("Servicce") at the class def but not at `new`, properties and
    // parameters ("Idd", "databaase") at declaration and constructor
    // parameter but not at `$this->` assignments, method names ("Deetails")
    // at the definition but not at the call. Keywords (class, function,
    // return, ...) and exception class names in throw/catch ("Excepton")
    // are not checked; neither are variable usages like `$dbb` or
    // `$resullt` inside `empty()`.
    assert_spelling_at(
        LanguageType::Php,
        sample_text,
        &[
            ("Servicez", &[0]),
            ("Servicce", &[0]),
            ("ACTIVVE", &[0]),
            ("Idd", &[0, 1]),
            ("databaase", &[0, 1]),
            ("Deetails", &[0]),
            ("querry", &[0]),
            ("foundd", &[0]),
            ("formatt", &[0]),
            ("amountt", &[0]),
            ("symboll", &[0]),
            ("formattted", &[0]),
            ("errr", &[0]),
            ("userr", &[0]),
        ],
    );
}
