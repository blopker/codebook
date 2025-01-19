use std::path::PathBuf;
use std::sync::Arc;

use codebook::dictionary::{SpellCheckResult, TextRange};
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use codebook::Codebook;
use codebook_config::CodebookConfig;
use log::info;

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub codebook: Codebook,
    pub config: Arc<CodebookConfig>,
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
                // hover_provider: Some(HoverProviderCapability::Simple(true)),
                // inlay_hint_provider: Some(OneOf::Left(true)),
                // code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "Codebook Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
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
        self.publish_spellcheck_diagnostics(&params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Clear diagnostics when a file is closed.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.publish_spellcheck_diagnostics(&params.text_document.uri, &text)
                .await;
        }
    }

    // async fn hover(&self, params: HoverParams) -> RpcResult<Option<Hover>> {
    //     let contents = HoverContents::Scalar(MarkedString::String("Hello, world!".to_string()));
    //     Ok(Some(Hover {
    //         contents,
    //         range: None,
    //     }))
    // }
}

impl Backend {
    pub fn new(client: Client, cache_dir: &PathBuf, workspace_dir: &PathBuf) -> Self {
        let mut config =
            CodebookConfig::load_from_dir(workspace_dir).expect("Unable to make config.");
        config.cache_dir = cache_dir.clone();
        let config_arc = Arc::new(config);
        let codebook = Codebook::new(Arc::clone(&config_arc)).expect("Unable to make codebook");
        Self {
            client,
            codebook,
            config: Arc::clone(&config_arc),
        }
    }
    fn make_diagnostic(&self, result: &SpellCheckResult, range: &TextRange) -> Diagnostic {
        let suggestion_message = match result.suggestions.len() {
            0 => "No suggestions found.".to_string(),
            _ => format!("Suggestions: {}", result.suggestions.join(", ")),
        };
        let message = format!(
            "Possible spelling error: '{}'. {}",
            result.word, suggestion_message
        );
        Diagnostic {
            range: Range {
                start: Position {
                    line: range.start_line,
                    character: range.start_char,
                },
                end: Position {
                    line: range.end_line,
                    character: range.end_char,
                },
            },
            severity: Some(DiagnosticSeverity::INFORMATION),
            code: None,
            code_description: None,
            source: Some("Codebook".to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Helper method to publish diagnostics for spell-checking.
    async fn publish_spellcheck_diagnostics(&self, uri: &Url, text: &str) {
        let _ = self.config.reload();
        // Convert the file URI to a local file path (if needed).
        let uri = uri.clone();
        let file_path = uri.to_file_path().unwrap_or_default();
        info!("Spell-checking file: {:?}", file_path);
        // 1) Perform spell-check (stubbed function below).
        let spell_results = self.spell_check(file_path.to_str().unwrap_or_default(), text);

        // 2) Convert the results to LSP diagnostics.
        let diagnostics: Vec<Diagnostic> = spell_results
            .into_iter()
            .flat_map(|res| {
                // For each misspelling, create a diagnostic for each location.
                let mut new_locations = vec![];
                for loc in &res.locations {
                    let diagnostic = self.make_diagnostic(&res, loc);
                    new_locations.push(diagnostic);
                }
                new_locations
            })
            .collect();

        info!("Diagnostics: {:?}", diagnostics);
        // 3) Send the diagnostics to the client.
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
        info!("Published diagnostics for: {:?}", file_path);
    }

    fn spell_check(
        &self,
        file_name: &str,
        file_contents: &str,
    ) -> Vec<codebook::dictionary::SpellCheckResult> {
        self.codebook
            .dictionary
            .spell_check_file_memory(file_name, file_contents)
    }
}
