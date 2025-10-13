use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_zig_simple() {
    utils::init_logging();
    let sample_text = r#"
const tesst = 5;
var valuue = 10;
"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Zig), None)
        .to_vec();

    println!("Misspelled words: {:#?}", misspelled);

    assert!(misspelled.iter().any(|w| w.word == "tesst"));
    assert!(misspelled.iter().any(|w| w.word == "valuue"));
}

#[test]
fn test_zig_strings() {
    utils::init_logging();
    let sample_text = r#"
test "bad speling" {
    const msg = "Hello Wolrd";
}
"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Zig), None)
        .to_vec();

    println!("Misspelled words: {:#?}", misspelled);

    assert!(misspelled.iter().any(|w| w.word == "speling"));
    assert!(misspelled.iter().any(|w| w.word == "Wolrd"));
}

#[test]
fn test_zig_functions() {
    utils::init_logging();
    let sample_text = r#"
fn addNumberrs(firstt: i32, seconnd: i32) i32 {
    return firstt + seconnd;
}
"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Zig), None)
        .to_vec();

    println!("Misspelled words: {:#?}", misspelled);
    assert!(misspelled.iter().any(|w| w.word == "Numberrs"));
    assert!(misspelled.iter().any(|w| w.word == "firstt"));
    assert!(misspelled.iter().any(|w| w.word == "seconnd"));
}
