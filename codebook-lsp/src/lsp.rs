use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use codebook::dictionary::{SpellCheckResult, TextRange};
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
//Hoverz
use codebook::Codebook;
use codebook_config::CodebookConfig;
use log::info;

use crate::file_cache::{TextDocumentCache, TextDocumentCacheItem};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub codebook: Codebook,
    pub config: Arc<CodebookConfig>,
    document_cache: Arc<RwLock<TextDocumentCache>>,
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
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
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

    async fn code_action(&self, params: CodeActionParams) -> RpcResult<Option<CodeActionResponse>> {
        let mut actions: Vec<CodeActionOrCommand> = vec![];
        let doc = match self.get_cache(&params.text_document.uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        params.context.diagnostics.iter().for_each(|diag| {
            let word = &doc.lines[diag.range.start.line as usize]
                [diag.range.start.character as usize..diag.range.end.character as usize];
            let suggestions = self.codebook.dictionary.suggest(word);
            suggestions.iter().for_each(|suggestion| {
                actions.push(CodeActionOrCommand::CodeAction(self.make_suggestion(
                    suggestion,
                    &diag.range,
                    &params.text_document.uri,
                )));
            });
        });
        Ok(Some(actions))
    }
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
            document_cache: Arc::new(RwLock::new(TextDocumentCache::new())),
        }
    }
    fn make_diagnostic(&self, result: &SpellCheckResult, range: &TextRange) -> Diagnostic {
        let message = format!("Possible spelling issue: '{}'.", result.word);
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

    fn make_suggestion(&self, suggestion: &str, range: &Range, uri: &Url) -> CodeAction {
        let title = format!("Replace with {}", suggestion);
        let mut map = HashMap::new();
        map.insert(
            uri.clone(),
            vec![TextEdit {
                range: range.clone(),
                new_text: suggestion.to_string(),
            }],
        );
        let edit = Some(WorkspaceEdit {
            changes: Some(map),
            document_changes: None,
            change_annotations: None,
        });
        CodeAction {
            title: title.to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit,
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        }
    }

    fn get_cache(&self, uri: &Url) -> Option<TextDocumentCacheItem> {
        self.document_cache
            .write()
            .unwrap()
            .get(&uri.to_string())
            .cloned()
    }

    fn update_cache(&self, uri: &Url, text: &str) {
        self.document_cache.write().unwrap().insert(
            uri.to_string(),
            TextDocumentCacheItem::new(&uri.as_str(), 0, "no", &text),
        );
    }

    /// Helper method to publish diagnostics for spell-checking.
    async fn publish_spellcheck_diagnostics(&self, uri: &Url, text: &str) {
        let _ = self.config.reload();
        self.update_cache(uri, text);
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
