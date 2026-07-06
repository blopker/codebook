use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_typescript_location() {
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
    // Keywords, built-ins (Promise, console, fetch), and imported names are
    // not flagged; exact set equality guards that.
    assert_spelling_at(
        LanguageType::Typescript,
        sample_text,
        &[
            // Flagged at the interface definition; the two `UserProifle`
            // type references are not.
            ("Proifle", &[0]),
            ("Adress", &[0]),
            ("Acttive", &[0]),
            ("Managger", &[0]),
            ("userz", &[0]),
            // Flagged at the constructor parameter definition; the
            // `this.apiEndpoont` usage is not.
            ("Endpoont", &[0]),
            ("Usars", &[0]),
            // Flagged at the declaration; the `respoonse.json()` usage is not.
            ("respoonse", &[0]),
            // Flagged at the catch parameter; the usage in the log call is not.
            ("erorr", &[0]),
            // String contents are checked.
            ("usars", &[0]),
            ("failled", &[0]),
        ],
    );
}
