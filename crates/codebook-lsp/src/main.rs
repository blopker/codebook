mod file_cache;
mod init_options;
mod lint;
mod lsp;
mod lsp_logger;

use clap::{Parser, Subcommand};
use codebook_config::CodebookConfigFile;
use log::{LevelFilter, debug, info};
use lsp::Backend;
use lsp_logger::LspLogger;
use std::env;
use std::path::{Path, PathBuf};
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
    /// Check files for spelling errors
    Lint {
        /// Files or glob patterns to spell-check
        #[arg(required = true)]
        files: Vec<String>,
        /// Only report each misspelled word once, ignoring duplicates across files
        #[arg(short = 'u', long)]
        unique: bool,
    },
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
    debug!("Logger initialized with log level: {log_level:?}");
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
            let config = CodebookConfigFile::default();
            info!("Cleaning: {:?}", config.cache_dir);
            config.clean_cache()
        }
        Some(Commands::Lint { files, unique }) => {
            if lint::run_lint(files, root, *unique) {
                std::process::exit(1);
            }
        }
        None => {}
    }
}

async fn serve_lsp(root: &Path) {
    let version = env!("CARGO_PKG_VERSION");
    let build_profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    info!("Starting Codebook Language Server v{version}-{build_profile}...");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let inner_root = root.to_owned();
    let (service, socket) = LspService::new(|client| Backend::new(client, &inner_root));
    Server::new(stdin, stdout, socket).serve(service).await;
}
