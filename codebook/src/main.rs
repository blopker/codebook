use codebook::Codebook;
use std::env;
use std::path::Path;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Arc::new(codebook_config::CodebookConfig::new_no_file());
    let processor = Codebook::new(config).unwrap();

    // println!("My path is {:?}", args);
    if args.len() < 2 {
        let sample_text = r#"
            fn calculate_user_age(birthDate: String) -> u32 {
                // This is an example_function that calculates age
                let userAge = get_current_date() - birthDate;
                userAge
            }
        "#;

        let misspelled = processor.dictionary.spell_check(sample_text, "rust");
        println!("Misspelled words: {:?}", misspelled);
        return;
    }

    let path = Path::new(args[1].as_str());
    if !path.exists() {
        eprintln!("Can't find file {path:?}");
        return;
    }
    let results = processor
        .dictionary
        .spell_check_file(path.to_str().unwrap());
    println!("Misspelled words: {:?}", results);
    println!("Done");
}
