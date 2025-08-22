use log::{Level, LevelFilter, Log, Metadata, Record};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use tokio::sync::mpsc::{self, Sender};
use tower_lsp::Client;
use tower_lsp::lsp_types::MessageType;

const BUFFER_SIZE: usize = 1000;

pub struct LspLogger {
    sender: Mutex<Option<Sender<LogMessage>>>,
    level: Mutex<LevelFilter>,
    // Buffer for storing logs before LSP client is available
    buffer: Mutex<VecDeque<LogMessage>>,
}

struct LogMessage {
    level: Level,
    message: String,
}

impl LogMessage {
    fn clone(&self) -> Self {
        LogMessage {
            level: self.level,
            message: self.message.clone(),
        }
    }
}

// Global static logger instance
static LOGGER: OnceLock<&'static LspLogger> = OnceLock::new();

impl LspLogger {
    /// Initialize the logger early without an LSP client
    /// Logs will be sent to stderr and buffered
    pub fn init_early(level: LevelFilter) -> Result<(), log::SetLoggerError> {
        let logger = Box::leak(Box::new(LspLogger {
            sender: Mutex::new(None),
            level: Mutex::new(level),
            buffer: Mutex::new(VecDeque::with_capacity(BUFFER_SIZE)),
        }));

        // Try to set the logger, ignore if already initialized
        let _ = LOGGER.set(logger);

        // Get the logger (either the one we just set or the existing one)
        let logger = LOGGER.get().expect("Logger should be initialized");
        log::set_logger(*logger).map(|()| log::set_max_level(level))
    }

    /// Attach the LSP client to the logger and flush buffered logs
    pub fn attach_client(client: Client, level: LevelFilter) {
        if let Some(logger) = LOGGER.get() {
            // Update log level
            *logger.level.lock().unwrap() = level;
            log::set_max_level(level);

            // Create a channel for log messages
            let (sender, mut receiver) = mpsc::channel::<LogMessage>(BUFFER_SIZE);

            // Get buffered messages before we update the sender
            let buffered_messages: Vec<LogMessage> = {
                let buffer = logger.buffer.lock().unwrap();
                buffer.iter().map(|msg| msg.clone()).collect()
            };

            // Update the sender
            *logger.sender.lock().unwrap() = Some(sender.clone());

            // Set up a background task to send logs to LSP client
            let client_clone = client.clone();
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

            // Flush buffered messages to the LSP client
            for msg in buffered_messages {
                // Send to the channel, ignore if it fails
                let _ = sender.try_send(msg);
            }

            // Clear the buffer since we've sent everything
            logger.buffer.lock().unwrap().clear();
        }
    }
}

impl Log for LspLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= *self.level.lock().unwrap()
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

        let log_msg = LogMessage {
            level: record.level(),
            message: message.clone(),
        };

        // If we have a sender, use it
        let sender_guard = self.sender.lock().unwrap();
        if let Some(sender) = sender_guard.as_ref() {
            // Send log message to channel, ignore if channel is full
            let _ = sender.try_send(log_msg);
        } else {
            // No LSP client yet, log to stderr and buffer
            eprintln!("{}", message);

            // Drop the sender guard before locking buffer to avoid potential deadlock
            drop(sender_guard);

            // Add to buffer
            let mut buffer = self.buffer.lock().unwrap();
            if buffer.len() >= BUFFER_SIZE {
                // Remove oldest message if buffer is full
                buffer.pop_front();
            }
            buffer.push_back(log_msg);
        }
    }

    fn flush(&self) {}
}
