mod file_cache;
mod lsp;
mod lsp_logger;

use clap::{Parser, Subcommand};
use codebook_config::CodebookConfig;
use log::{LevelFilter, info};
use lsp::Backend;
use lsp_logger::LspLogger;
use std::env;
use std::path::{Path, PathBuf};
use tokio::task;
use tower_lsp::{LspService, Server};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    // Initialize logger early with stderr output and buffering
    // Default to INFO level, will be adjusted when LSP client connects
    let log_level = match env::var("RUST_LOG").as_deref() {
        Ok("debug") => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };
    LspLogger::init_early(log_level).expect("Failed to initialize early logger");

    let cli = Cli::parse();

    let root = match cli.root.as_deref() {
        Some(path) => path,
        None => Path::new("."),
    };

    match &cli.command {
        Some(Commands::Serve {}) => {
            serve_lsp(root).await;
        }
        Some(Commands::Clean {}) => {
            let config = CodebookConfig::default();
            info!("Cleaning: {:?}", config.cache_dir);
            config.clean_cache()
        }
        None => {}
    }
}

async fn serve_lsp(root: &Path) {
    info!("Starting Codebook Language Server..");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let inner_root = root.to_owned();

    // Some blocking setup is done, so spawn_block!
    let (service, socket) =
        task::spawn_blocking(move || LspService::new(|client| Backend::new(client, &inner_root)))
            .await
            .unwrap();

    Server::new(stdin, stdout, socket).serve(service).await;
}
