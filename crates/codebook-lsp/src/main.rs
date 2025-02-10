mod file_cache;
mod lsp;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use codebook_config::CodebookConfig;
use log::info;
use lsp::Backend;
use tower_lsp::{LspService, Server};

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Root of the workspace/project being checked.
    /// This may or may not have a codebook.toml file.
    #[arg(short, long, value_name = "FOLDER")]
    root: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve the Language Server
    Serve {},
    /// Remove server cache
    Clean {},
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize logging so we can see server messages in the console.
    env_logger::init();
    let cli = Cli::parse();

    let root = match cli.root.as_deref() {
        Some(path) => path,
        None => Path::new("."),
    };

    match &cli.command {
        Some(Commands::Serve {}) => {
            serve_lsp(&root.to_path_buf()).await;
        }
        Some(Commands::Clean {}) => {
            let config = CodebookConfig::new_no_file();
            info!("Cleaning: {:?}", config.cache_dir);
            config.clean_cache()
        }
        None => {}
    }
}

async fn serve_lsp(root: &PathBuf) {
    info!("Starting Codebook Language Server...");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| Backend::new(client, root));
    Server::new(stdin, stdout, socket).serve(service).await;
}
