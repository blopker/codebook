mod file_cache;
mod init_options;
mod lint;
mod lsp;
mod lsp_logger;

use clap::{Parser, Subcommand};
use codebook_config::{CodebookConfig, CodebookConfigFile};
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
        /// Show spelling suggestions for each misspelled word
        #[arg(short = 's', long)]
        suggest: bool,
    },
    /// Add words to the dictionary
    Add {
        /// Words to add to the allowlist
        #[arg(required = true)]
        words: Vec<String>,
        /// Add to the global config instead of the project config
        #[arg(short, long)]
        global: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();

    // Initialize logger early with stderr output and buffering.
    // Default to INFO for LSP, WARN for lint (to suppress LSP-oriented noise).
    let is_lint = matches!(cli.command, Some(Commands::Lint { .. }));
    let log_level = match env::var("RUST_LOG").as_deref() {
        Ok("debug") => LevelFilter::Debug,
        Ok("info") => LevelFilter::Info,
        _ if is_lint => LevelFilter::Warn,
        _ => LevelFilter::Info,
    };
    LspLogger::init_early(log_level).expect("Failed to initialize early logger");
    debug!("Logger initialized with log level: {log_level:?}");

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
        Some(Commands::Lint {
            files,
            unique,
            suggest,
        }) => {
            let code = match lint::run_lint(files, root, *unique, *suggest) {
                lint::LintResult::Clean => 0,
                lint::LintResult::Errors => 1,
                lint::LintResult::Failure => 2,
            };
            std::process::exit(code);
        }
        Some(Commands::Add { words, global }) => {
            if let Err(e) = add_words(root, words, *global) {
                eprintln!("error: {e}");
                std::process::exit(2);
            }
        }
        None => {}
    }
}

/// Adds words to the project (or global) config's allowlist and saves the file,
/// creating it if it doesn't exist yet.
fn add_words(root: &Path, words: &[String], global: bool) -> Result<(), std::io::Error> {
    let config = CodebookConfigFile::load(Some(root))?;
    let mut added = 0;
    for word in words {
        let inserted = if global {
            config.add_word_global(word)?
        } else {
            config.add_word(word)?
        };
        if inserted {
            added += 1;
        } else {
            println!("'{word}' is already in the dictionary");
        }
    }

    if added == 0 {
        return Ok(());
    }
    let path = if global {
        config.save_global()?;
        config.global_config_path()
    } else {
        config.save()?;
        config.project_config_path()
    };
    match path {
        Some(p) => println!("Added {added} word(s) to {}", p.display()),
        None => println!("Added {added} word(s)"),
    }
    Ok(())
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
