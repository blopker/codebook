use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_csharp_simple() {
    utils::init_logging();
    let sample_text = r#"
class Demo {
    const int tesst = 5;

    static void Run() {
        var valuue = 10;
    }
}
"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::CSharp), None)
        .to_vec();

    println!("Misspelled words: {:#?}", misspelled);

    assert!(misspelled.iter().any(|w| w.word == "tesst"));
    assert!(misspelled.iter().any(|w| w.word == "valuue"));
}

#[test]
fn test_csharp_strings_and_comments() {
    utils::init_logging();
    let sample_text = r#"
// Comment with speling error.

/* Multi-line comment
with error on seconnd line */

class DemoStrings {
    static void Test() {
        var msg = "Hello Wolrd"; // inline coment with eror
        System.Console.WriteLine(msg);

        var interpolated = $"Helo {msg}";
        System.Console.WriteLine(interpolated);

        var multi = @"ernor
           spelingg
        "); 
        System.Console.WriteLine(multi);
    }
}
"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::CSharp), None)
        .to_vec();

    println!("Misspelled words: {:#?}", misspelled);

    assert!(misspelled.iter().any(|w| w.word == "speling"));
    assert!(misspelled.iter().any(|w| w.word == "seconnd"));
    assert!(misspelled.iter().any(|w| w.word == "Wolrd"));
    assert!(misspelled.iter().any(|w| w.word == "coment"));
    assert!(misspelled.iter().any(|w| w.word == "eror"));
    assert!(misspelled.iter().any(|w| w.word == "Helo"));
    assert!(misspelled.iter().any(|w| w.word == "ernor"));
    assert!(misspelled.iter().any(|w| w.word == "spelingg"));
}

#[test]
fn test_csharp_functions() {
    utils::init_logging();
    let sample_text = r#"
class MathUtil {
    static int AddNumberrs(int firstt, int seconnd) {
        int resullt = firstt + seconnd;
        return resullt;
    }
}
"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::CSharp), None)
        .to_vec();

    println!("Misspelled words: {:#?}", misspelled);

    assert!(misspelled.iter().any(|w| w.word == "Numberrs"));
    assert!(misspelled.iter().any(|w| w.word == "firstt"));
    assert!(misspelled.iter().any(|w| w.word == "seconnd"));
    assert!(misspelled.iter().any(|w| w.word == "resullt"));
}
