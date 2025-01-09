mod lsp;
use log::info;
use lsp::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logging so we can see server messages in the console.
    env_logger::init();
    info!("Starting SpellCheck Language Server...");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
