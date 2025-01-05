mod code_dictionary;
mod downloader;
mod lsp;
mod queries;
mod splitter;
use code_dictionary::CodeDictionary;
use downloader::DictionaryDownloader;
use log::info;
use lsp::Backend;
use std::env;
use std::path::Path;
use tower_lsp::{LspService, Server};

fn main() {
    let args: Vec<String> = env::args().collect();
    let downloader =
        DictionaryDownloader::new(downloader::DEFAULT_BASE_URL, "../.cache/dictionaries");
    let files = downloader.get("en").unwrap();
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
    if args[1].as_str() == "server" {
        start_server();
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

#[tokio::main(flavor = "current_thread")]
async fn start_server() {
    // Initialize logging so we can see server messages in the console.
    env_logger::init();
    info!("Starting SpellCheck Language Server...");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
