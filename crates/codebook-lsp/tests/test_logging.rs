use codebook_lsp::lsp_logger::LspLogger;
use log::{LevelFilter, info};
use std::path::Path;

#[tokio::main]
async fn main() {
    // Initialize the logger early - logs will go to stderr and be buffered
    LspLogger::init_early(LevelFilter::Info).expect("Failed to initialize logger");

    info!("Early logging test started");
    info!("This message should appear on stderr immediately");

    // Create a dummy client (this would normally come from the LSP)
    // For testing, we'll just simulate the backend creation which triggers downloader
    let _workspace_dir = Path::new(".");

    info!("About to create Backend - this will trigger Downloader creation");

    // Note: We can't actually create a real Backend without a proper Client
    // but this example shows the structure

    // In real usage:
    // 1. LspLogger::init_early() is called in main()
    // 2. Backend::new() is called, which creates Codebook/DictionaryManager/Downloader
    // 3. Downloader logs "Cache folder at: ..." which goes to stderr and is buffered
    // 4. Later, initialize() is called and LspLogger::attach_client() is invoked
    // 5. All buffered logs are flushed to the LSP client

    info!("In production, the flow would be:");
    info!("1. Early logger initialization (stderr + buffering)");
    info!("2. Backend creation triggers Downloader logs");
    info!("3. LSP client attachment flushes buffered logs");

    // Simulate some more logging that would be buffered
    for i in 1..=5 {
        info!("Buffered log message #{}", i);
    }

    info!("Test complete - all logs should have appeared on stderr");
}
