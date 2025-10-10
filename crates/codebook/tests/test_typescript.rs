use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_typescript_location() {
    utils::init_logging();
    let sample_text = r#"
    import { Component } from 'react';

    interface UserProifle {
        id: number;
        firstName: string;
        lastName: string;
        emailAdress: string;
        isActtive: boolean;
    }

    class UserManagger extends Component {
        private userz: UserProifle[] = [];

        constructor(private apiEndpoont: string) {
            super();
        }

        public async fetchUsars(): Promise<UserProifle[]> {
            try {
                const respoonse = await fetch(this.apiEndpoont);
                return await respoonse.json();
            } catch (erorr) {
                console.log("Fetching usars failled:", erorr);
                return [];
            }
        }
    }"#;

    let expected = vec![
        WordLocation::new(
            "Proifle".to_string(),
            vec![TextRange {
                start_byte: 59,
                end_byte: 66,
            }],
        ),
        WordLocation::new(
            "Adress".to_string(),
            vec![TextRange {
                start_byte: 155,
                end_byte: 161,
            }],
        ),
        WordLocation::new(
            "Acttive".to_string(),
            vec![TextRange {
                start_byte: 181,
                end_byte: 188,
            }],
        ),
        WordLocation::new(
            "Managger".to_string(),
            vec![TextRange {
                start_byte: 220,
                end_byte: 228,
            }],
        ),
        WordLocation::new(
            "userz".to_string(),
            vec![TextRange {
                start_byte: 265,
                end_byte: 270,
            }],
        ),
        WordLocation::new(
            "Endpoont".to_string(),
            vec![TextRange {
                start_byte: 324,
                end_byte: 332,
            }],
        ),
        WordLocation::new(
            "Usars".to_string(),
            vec![TextRange {
                start_byte: 402,
                end_byte: 407,
            }],
        ),
        WordLocation::new(
            "respoonse".to_string(),
            vec![TextRange {
                start_byte: 476,
                end_byte: 485,
            }],
        ),
        WordLocation::new(
            "erorr".to_string(),
            vec![TextRange {
                start_byte: 587,
                end_byte: 592,
            }],
        ),
        WordLocation::new(
            "usars".to_string(),
            vec![TextRange {
                start_byte: 634,
                end_byte: 639,
            }],
        ),
        WordLocation::new(
            "failled".to_string(),
            vec![TextRange {
                start_byte: 640,
                end_byte: 647,
            }],
        ),
    ];

    let not_expected = [
        "import",
        "Component",
        "react",
        "interface",
        "number",
        "string",
        "boolean",
        "class",
        "extends",
        "private",
        "constructor",
        "super",
        "public",
        "async",
        "Promise",
        "try",
        "const",
        "await",
        "fetch",
        "return",
        "json",
        "catch",
        "console",
        "log",
    ];

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Typescript), None)
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
