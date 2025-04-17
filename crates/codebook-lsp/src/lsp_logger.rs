use log::{Level, LevelFilter, Log, Metadata, Record};
use tokio::sync::mpsc::{self, Sender};
use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;

pub struct LspLogger {
    sender: Sender<LogMessage>,
    level: LevelFilter,
}

struct LogMessage {
    level: Level,
    message: String,
}

impl LspLogger {
    pub fn init(client_clone: Client, level: LevelFilter) -> Result<(), log::SetLoggerError> {
        // Create a channel for log messages
        let (sender, mut receiver) = mpsc::channel::<LogMessage>(100);

        // Create a new logger
        let logger = Self { sender, level };

        // Set up a background task to send logs to LSP client
        tokio::spawn(async move {
            while let Some(log_msg) = receiver.recv().await {
                let lsp_level = match log_msg.level {
                    Level::Error => MessageType::ERROR,
                    Level::Warn => MessageType::WARNING,
                    Level::Info => MessageType::INFO,
                    Level::Debug | Level::Trace => MessageType::LOG,
                };

                // Ignore any errors from sending log messages
                let _ = client_clone.log_message(lsp_level, log_msg.message).await;
            }
        });

        // Register our custom logger
        log::set_boxed_logger(Box::new(logger)).map(|()| log::set_max_level(level))
    }
}

impl Log for LspLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Format the log message with module path
        let message = format!(
            "[{}] {}: {}",
            record.target(),
            record.level(),
            record.args()
        );

        // Send log message to channel, ignore if channel is full
        let _ = self.sender.try_send(LogMessage {
            level: record.level(),
            message,
        });
    }

    fn flush(&self) {}
}
