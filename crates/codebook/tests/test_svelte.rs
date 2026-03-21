use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_svelte_html() {
    utils::init_logging();
    let sample_text = r#"
        <main>
        <h1>Welcom to my app</h1>
        <p>Enjoye your stay, pleaze report bugs.</p>
        </main>"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Svelte), None)
        .to_vec();
    println!("Misspelled: {misspelled:#?}");

    assert_eq!(misspelled.len(), 3);
    assert!(misspelled.iter().any(|w| w.word == "Welcom"));
    assert!(misspelled.iter().any(|w| w.word == "Enjoye"));
    assert!(misspelled.iter().any(|w| w.word == "pleaze"));
}

#[test]
fn test_svelte_script() {
    utils::init_logging();
    let sample_text = r#"
        <script>
        const mesage = "Helo Wrold";
        let naeme = 'Welcom back';
        </script>"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Svelte), None)
        .to_vec();
    println!("Misspelled: {misspelled:#?}");

    assert_eq!(misspelled.len(), 5);
    assert!(misspelled.iter().any(|w| w.word == "mesage"));
    assert!(misspelled.iter().any(|w| w.word == "Helo"));
    assert!(misspelled.iter().any(|w| w.word == "Wrold"));
    assert!(misspelled.iter().any(|w| w.word == "naeme"));
    assert!(misspelled.iter().any(|w| w.word == "Welcom"));
}

#[test]
fn test_svelte_style() {
    utils::init_logging();
    let sample_text = r#"
        <style>
        .card {
            backgrond-color: #fff;
            font-wieght: bold;
            bordre: 1px solid #ccc;
        }
        </style>"#;

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::Svelte), None)
        .to_vec();
    println!("Misspelled: {misspelled:#?}");

    assert_eq!(misspelled.len(), 3);
    assert!(misspelled.iter().any(|w| w.word == "backgrond"));
    assert!(misspelled.iter().any(|w| w.word == "wieght"));
    assert!(misspelled.iter().any(|w| w.word == "bordre"));
}
