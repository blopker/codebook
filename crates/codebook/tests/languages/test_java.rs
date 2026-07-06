use codebook::queries::LanguageType;

use super::utils::assert_spelling_at;

#[test]
fn test_java_location() {
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
    // Keywords and method-call references (`recoveryMthod`) are not
    // spell-checked; exact set equality guards that.
    assert_spelling_at(
        LanguageType::Java,
        sample_text,
        &[
            ("Singl", &[0]),
            ("Blck", &[0]),
            // Flagged at the interface definition; the `implements` reference
            // is not.
            ("Exampl", &[0]),
            // Flagged at both method definitions (interface and class).
            ("Somethng", &[0, 1]),
            ("Statuss", &[0]),
            ("ACTIV", &[0]),
            ("Soem", &[0]),
            ("messag", &[0]),
            // Flagged at the parameter definition; the usage in the string
            // concatenation is not.
            ("smth", &[0]),
            // Flagged at the catch parameter; the usage in println is not.
            ("errorr", &[0]),
        ],
    );
}
