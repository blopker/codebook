mod lsp;
use codebook::downloader::{self, DictionaryDownloader};
use codebook::CodeDictionary;
use log::info;
use lsp::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logging so we can see server messages in the console.
    env_logger::init();
    info!("Starting SpellCheck Language Server...");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let downloader =
        DictionaryDownloader::new(downloader::DEFAULT_BASE_URL, "../.cache/dictionaries");
    let files = downloader.get("en").unwrap();
    let processor = CodeDictionary::new(&files.aff_local_path, &files.dic_local_path).unwrap();
    let (service, socket) = LspService::new(|client| Backend { client, processor });
    Server::new(stdin, stdout, socket).serve(service).await;
}
