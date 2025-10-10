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
                start_byte: 18,
                end_byte: 25,
            }],
        ),
        WordLocation::new(
            "Adress".to_string(),
            vec![TextRange {
                start_byte: 13,
                end_byte: 19,
            }],
        ),
        WordLocation::new(
            "Acttive".to_string(),
            vec![TextRange {
                start_byte: 10,
                end_byte: 17,
            }],
        ),
        WordLocation::new(
            "Managger".to_string(),
            vec![TextRange {
                start_byte: 14,
                end_byte: 22,
            }],
        ),
        WordLocation::new(
            "userz".to_string(),
            vec![TextRange {
                start_byte: 16,
                end_byte: 21,
            }],
        ),
        WordLocation::new(
            "Endpoont".to_string(),
            vec![TextRange {
                start_byte: 31,
                end_byte: 39,
            }],
        ),
        WordLocation::new(
            "Usars".to_string(),
            vec![TextRange {
                start_byte: 26,
                end_byte: 31,
            }],
        ),
        WordLocation::new(
            "respoonse".to_string(),
            vec![TextRange {
                start_byte: 22,
                end_byte: 31,
            }],
        ),
        WordLocation::new(
            "erorr".to_string(),
            vec![TextRange {
                start_byte: 21,
                end_byte: 26,
            }],
        ),
        WordLocation::new(
            "usars".to_string(),
            vec![TextRange {
                start_byte: 38,
                end_byte: 43,
            }],
        ),
        WordLocation::new(
            "failled".to_string(),
            vec![TextRange {
                start_byte: 44,
                end_byte: 51,
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
