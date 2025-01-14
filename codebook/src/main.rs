mod downloader;
use codebook::CodeDictionary;
use downloader::DictionaryDownloader;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let loader = DictionaryDownloader::new(downloader::DEFAULT_BASE_URL, "../.cache/dictionaries");
    let files = loader.get("en").unwrap();
    let processor = CodeDictionary::new(&files.aff_local_path, &files.dic_local_path).unwrap();

    // println!("My path is {:?}", args);
    if args.len() < 2 {
        let sample_text = r#"
            fn calculate_user_age(birthDate: String) -> u32 {
                // This is an example_function that calculates age
                let userAge = get_current_date() - birthDate;
                userAge
            }
        "#;

        let misspelled = processor.spell_check(sample_text, "rust");
        println!("Misspelled words: {:?}", misspelled);
        return;
    }

    let path = Path::new(args[1].as_str());
    if !path.exists() {
        eprintln!("Can't find file {path:?}");
        return;
    }
    let results = processor.spell_check_file(path.to_str().unwrap());
    println!("Misspelled words: {:?}", results);
    println!("Done");
}
