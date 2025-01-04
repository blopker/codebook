mod code_dictionary;
mod downloader;
mod queries;
mod splitter;
use code_dictionary::CodeDictionary;
use downloader::DictionaryDownloader;
use std::env;
use std::path::Path;

fn main() {
    let downloader =
        DictionaryDownloader::new(downloader::DEFAULT_BASE_URL, "../.cache/dictionaries");
    let files = downloader.get("en").unwrap();
    let processor = CodeDictionary::new(&files.aff_local_path, &files.dic_local_path).unwrap();
    let args: Vec<String> = env::args().collect();

    println!("My path is {:?}", args);
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
//
// fn main() {
//     let stdin = io::stdin();
//     let mut stdout = io::stdout();

//     for line in stdin.lock().lines() {
//         if let Ok(json_rpc) = line {
//             // Parse JSON-RPC message here
//             match parse_json_rpc(&json_rpc) {
//                 Some(message) => handle_lsp_method(message, &mut stdout),
//                 None => eprintln!("Invalid JSON-RPC message"),
//             }
//         }
//     }
// }

// fn parse_json_rpc(json: &str) -> Option<String> {
//     // Implement your own parser or use a third-party library (like serde_json).
//     // For simplicity, this example does not include parsing.
//     eprintln!("{:?}", json);
//     None
// }

// fn handle_lsp_method(message: String, stdout: &mut dyn Write) {
//     // Handle LSP methods and send responses to the client here.
//     // This example does not include any LSP functionality.
//     writeln!(stdout, "{message}").unwrap(); // Send an empty response.
// }
