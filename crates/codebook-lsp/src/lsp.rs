use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use codebook::parser::TextRange;
use codebook::parser::WordLocation;
use codebook::queries::LanguageType;
use serde_json::Value;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use codebook::Codebook;
use codebook_config::CodebookConfig;
use log::{debug, info};

use crate::file_cache::TextDocumentCache;

const SOURCE_NAME: &str = "Codebook";

pub struct Backend {
    pub client: Client,
    pub codebook: Codebook,
    pub config: Arc<CodebookConfig>,
    pub document_cache: TextDocumentCache,
}

enum CodebookCommand {
    AddWord,
    Unknown,
}

impl From<&str> for CodebookCommand {
    fn from(command: &str) -> Self {
        match command {
            "codebook.addWord" => CodebookCommand::AddWord,
            _ => CodebookCommand::Unknown,
        }
    }
}

impl From<CodebookCommand> for String {
    fn from(command: CodebookCommand) -> Self {
        match command {
            CodebookCommand::AddWord => "codebook.addWord".to_string(),
            CodebookCommand::Unknown => "codebook.unknown".to_string(),
        }
    }
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
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![CodebookCommand::AddWord.into()],
                    work_done_progress_options: Default::default(),
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: format!("{SOURCE_NAME} Language Server"),
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
        self.document_cache.insert(&params.text_document);
        self.spell_check(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_cache.remove(&params.text_document.uri);
        // Clear diagnostics when a file is closed.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.document_cache.update(&params.text_document.uri, &text);
            self.spell_check(&params.text_document.uri).await;
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> RpcResult<Option<CodeActionResponse>> {
        let mut actions: Vec<CodeActionOrCommand> = vec![];
        let doc = match self
            .document_cache
            .get(&params.text_document.uri.to_string())
        {
            Some(doc) => doc,
            None => return Ok(None),
        };

        params.context.diagnostics.iter().for_each(|diag| {
            let line = doc
                .text
                .lines()
                .nth(diag.range.start.line as usize)
                .unwrap_or_default();
            let start_char = diag.range.start.character as usize;
            let end_char = diag.range.end.character as usize;
            let word = if start_char < line.len() && end_char <= line.len() {
                &line[start_char..end_char]
            } else {
                return;
            };

            let suggestions = self.codebook.get_suggestions(word);
            suggestions.iter().for_each(|suggestion| {
                actions.push(CodeActionOrCommand::CodeAction(self.make_suggestion(
                    suggestion,
                    &diag.range,
                    &params.text_document.uri,
                )));
            });
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Add '{}' to dictionary", word),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: format!("Add '{}' to dictionary", word),
                    command: CodebookCommand::AddWord.into(),
                    arguments: Some(vec![word.to_string().into()]),
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
        });

        Ok(Some(actions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> RpcResult<Option<Value>> {
        match CodebookCommand::from(params.command.as_str()) {
            CodebookCommand::AddWord => {
                let words = params
                    .arguments
                    .iter()
                    .filter_map(|arg| arg.as_str().map(|s| s.to_string()));
                let updated = self.add_words(words);
                if updated {
                    let _ = self.config.save();
                    self.recheck_all().await;
                }
                Ok(None)
            }
            CodebookCommand::Unknown => Ok(None),
        }
    }
}

impl Backend {
    pub fn new(client: Client, workspace_dir: &PathBuf) -> Self {
        let config = CodebookConfig::load_from_dir(workspace_dir).expect("Unable to make config.");
        let config_arc = Arc::new(config);
        let codebook = Codebook::new(Arc::clone(&config_arc)).expect("Unable to make codebook");
        Self {
            client,
            codebook,
            config: Arc::clone(&config_arc),
            document_cache: TextDocumentCache::new(),
        }
    }
    fn make_diagnostic(&self, result: &WordLocation, range: &TextRange) -> Diagnostic {
        let message = format!("Possible spelling issue '{}'.", result.word);
        Diagnostic {
            range: Range {
                start: Position {
                    line: range.line,
                    character: range.start_char,
                },
                end: Position {
                    line: range.line,
                    character: range.end_char,
                },
            },
            severity: Some(DiagnosticSeverity::INFORMATION),
            code: None,
            code_description: None,
            source: Some(SOURCE_NAME.to_string()),
            message,
            related_information: None,
            tags: None,
            data: None,
        }
    }

    fn add_words(&self, words: impl Iterator<Item = String>) -> bool {
        let mut should_save = false;
        for word in words {
            match self.config.add_word(&word) {
                Ok(true) => {
                    should_save = true;
                }
                Ok(false) => {
                    log::info!("Word '{}' already exists in dictionary.", word);
                }
                Err(e) => {
                    log::error!("Failed to add word: {}", e);
                }
            }
        }
        should_save
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

    async fn recheck_all(&self) {
        let urls = self.document_cache.cached_urls();
        debug!("Rechecking documents: {:?}", urls);
        for url in urls {
            self.publish_spellcheck_diagnostics(&url).await;
        }
    }

    async fn spell_check(&self, uri: &Url) {
        let did_reload = match self.config.reload() {
            Ok(did_reload) => did_reload,
            Err(e) => {
                log::error!("Failed to reload config: {}", e);
                false
            }
        };

        let did_config_change = uri
            .to_file_path()
            .map(|path| {
                self.config
                    .config_path
                    .as_ref()
                    .map(|config_path| path == *config_path)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

        if did_reload || did_config_change {
            self.recheck_all().await;
        } else {
            self.publish_spellcheck_diagnostics(uri).await;
        }
    }

    /// Helper method to publish diagnostics for spell-checking.
    async fn publish_spellcheck_diagnostics(&self, uri: &Url) {
        let doc = match self.document_cache.get(&uri.to_string()) {
            Some(doc) => doc,
            None => return,
        };
        // Convert the file URI to a local file path.
        let file_path = doc.uri.to_file_path().unwrap_or_default();
        info!("Spell-checking file: {:?}", file_path);
        // 1) Perform spell-check.
        let lang_type = match doc.language_id.as_deref() {
            Some(lang) => Some(LanguageType::from_str(lang)),
            _ => None,
        };
        let spell_results = self.codebook.spell_check(
            &doc.text,
            lang_type,
            Some(file_path.to_str().unwrap_or_default()),
        );

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

        debug!("Diagnostics: {:?}", diagnostics);
        // 3) Send the diagnostics to the client.
        self.client
            .publish_diagnostics(doc.uri, diagnostics, None)
            .await;
        debug!("Published diagnostics for: {:?}", file_path);
    }
}
