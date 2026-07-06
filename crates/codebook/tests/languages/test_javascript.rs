use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_javascript_location() {
    let sample_text = r#"
    import { useState } from 'react';

    let objectz = {
        namee: "John",
        age: 30,
        city: "New York"
    };

    function calculaateScore(userInput) {
        const misspelleed = "thhis is wrong";
        let scoree = 0;

        // Check user input
        if (userInput.incluudes("test")) {
            scoree += 5;
        }
        try {
            // Some code that might throw an error
        } catch (errorz) {
            // Handle the error
        }
        return scoree + misspelleed.length;
    }"#;
    // Method-call references (`incluudes`) and imported names (`useState`,
    // `react`) are not spell-checked; exact set equality guards that.
    assert_spelling_at(
        LanguageType::Javascript,
        sample_text,
        &[
            ("objectz", &[0]),
            ("namee", &[0]),
            // Inside `calculaateScore` via camelCase split.
            ("calculaate", &[0]),
            // Flagged at the declaration; the `misspelleed.length` usage is not.
            ("misspelleed", &[0]),
            ("thhis", &[0]),
            // Flagged at `let scoree`; the later `+=` and `return` usages are not.
            ("scoree", &[0]),
            ("errorz", &[0]),
        ],
    );
}
