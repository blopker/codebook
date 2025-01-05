use tower_lsp::jsonrpc::{Error as RpcError, Result as RpcResult};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use log::{error, info};

#[derive(Clone, Debug)]
pub struct TextRange {
    pub start_line: u32,
    pub start_char: u32,
    pub end_line: u32,
    pub end_char: u32,
}

#[derive(Clone, Debug)]
pub struct SpellCheckResult {
    pub word: String,
    pub suggestions: Vec<String>,
    pub locations: Vec<TextRange>,
}

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> RpcResult<InitializeResult> {
        info!("Server initialized");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "SpellCheck Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Server ready");
    }

    async fn shutdown(&self) -> RpcResult<()> {
        info!("Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.publish_spellcheck_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        self.publish_spellcheck_diagnostics(uri).await;
    }
}

impl Backend {
    /// Helper method to publish diagnostics for spell-checking.
    async fn publish_spellcheck_diagnostics(&self, uri: Url) {
        // Convert the file URI to a local file path (if needed).
        let file_path = uri.to_file_path().unwrap_or_default();

        // 1) Perform spell-check (stubbed function below).
        let spell_results = spell_check(file_path.to_str().unwrap_or_default());

        // 2) Convert the results to LSP diagnostics.
        let diagnostics: Vec<Diagnostic> = spell_results
            .into_iter()
            .flat_map(|res| {
                // For each misspelling, create a diagnostic for each location.
                res.locations.into_iter().map(move |loc| Diagnostic {
                    range: Range {
                        start: Position {
                            line: loc.start_line,
                            character: loc.start_char,
                        },
                        end: Position {
                            line: loc.end_line,
                            character: loc.end_char,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: None,
                    code_description: None,
                    source: Some("SpellCheck".to_string()),
                    message: format!(
                        "Possible spelling error: '{}'. Suggestions: {:?}",
                        res.word, res.suggestions
                    ),
                    related_information: None,
                    tags: None,
                    data: None,
                })
            })
            .collect();

        // 3) Send the diagnostics to the client.
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

/// Stubbed spell-check function.
/// In a real-world scenario, this might parse the file, identify misspellings,
/// and provide suggestions.
fn spell_check(_file_name: &str) -> Vec<SpellCheckResult> {
    vec![
        SpellCheckResult {
            word: "exampel".to_string(),
            suggestions: vec!["example".to_string(), "sample".to_string()],
            locations: vec![TextRange {
                start_line: 0,
                start_char: 10,
                end_line: 0,
                end_char: 17,
            }],
        },
        SpellCheckResult {
            word: "lenguage".to_string(),
            suggestions: vec!["language".to_string()],
            locations: vec![TextRange {
                start_line: 2,
                start_char: 5,
                end_line: 2,
                end_char: 13,
            }],
        },
    ]
}
