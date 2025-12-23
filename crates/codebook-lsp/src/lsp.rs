use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;
use std::sync::{Arc, OnceLock, RwLock};

use codebook::parser::get_word_from_string;
use codebook::queries::LanguageType;
use string_offsets::AllConfig;
use string_offsets::Pos;
use string_offsets::StringOffsets;

use log::error;
use serde_json::Value;
use tokio::task;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use codebook::Codebook;
use codebook_config::{CodebookConfig, CodebookConfigFile};
use log::{debug, info};

use crate::file_cache::TextDocumentCache;
use crate::init_options::ClientInitializationOptions;
use crate::lsp_logger;

const SOURCE_NAME: &str = "Codebook";

/// Computes the relative path of a file from a workspace directory.
/// Returns the relative path if the file is within the workspace, otherwise returns the absolute path.
/// If `workspace_dir_canonical` is provided, skips canonicalizing the workspace directory (optimization).
fn compute_relative_path(
    workspace_dir: &Path,
    workspace_dir_canonical: Option<&Path>,
    file_path: &Path,
) -> String {
    let workspace_canonical = match workspace_dir_canonical {
        Some(dir) => dir.to_path_buf(),
        None => match workspace_dir.canonicalize() {
            Ok(dir) => dir,
            Err(err) => {
                info!("Could not canonicalize workspace directory. Error: {err}.");
                return file_path.to_string_lossy().to_string();
            }
        },
    };

    match file_path.canonicalize() {
        Ok(canon_file_path) => match canon_file_path.strip_prefix(&workspace_canonical) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => file_path.to_string_lossy().to_string(),
        },
        Err(_) => file_path.to_string_lossy().to_string(),
    }
}

pub struct Backend {
    client: Client,
    workspace_dir: PathBuf,
    /// Cached canonicalized workspace directory for efficient relative path computation
    workspace_dir_canonical: Option<PathBuf>,
    codebook: OnceLock<Arc<Codebook>>,
    config: OnceLock<Arc<CodebookConfigFile>>,
    document_cache: TextDocumentCache,
    initialize_options: RwLock<Arc<ClientInitializationOptions>>,
}

enum CodebookCommand {
    AddWord,
    AddWordGlobal,
    IgnoreFile,
    Unknown,
}

impl From<&str> for CodebookCommand {
    fn from(command: &str) -> Self {
        match command {
            "codebook.addWord" => CodebookCommand::AddWord,
            "codebook.addWordGlobal" => CodebookCommand::AddWordGlobal,
            "codebook.ignoreFile" => CodebookCommand::IgnoreFile,
            _ => CodebookCommand::Unknown,
        }
    }
}

impl From<CodebookCommand> for String {
    fn from(command: CodebookCommand) -> Self {
        match command {
            CodebookCommand::AddWord => "codebook.addWord".to_string(),
            CodebookCommand::AddWordGlobal => "codebook.addWordGlobal".to_string(),
            CodebookCommand::IgnoreFile => "codebook.ignoreFile".to_string(),
            CodebookCommand::Unknown => "codebook.unknown".to_string(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> RpcResult<InitializeResult> {
        // info!("Capabilities: {:?}", params.capabilities);
        let client_options = ClientInitializationOptions::from_value(params.initialization_options);
        info!("Client options: {:?}", client_options);

        // Attach the LSP client to the logger and flush buffered logs
        lsp_logger::LspLogger::attach_client(self.client.clone(), client_options.log_level);
        info!(
            "LSP logger attached to client with log level: {}",
            client_options.log_level
        );

        *self.initialize_options.write().unwrap() = Arc::new(client_options);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF16),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..TextDocumentSyncOptions::default()
                    },
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        CodebookCommand::AddWord.into(),
                        CodebookCommand::AddWordGlobal.into(),
                        CodebookCommand::IgnoreFile.into(),
                    ],
                    work_done_progress_options: Default::default(),
                }),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![CodeActionKind::QUICKFIX]),
                        resolve_provider: None,
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: format!("{SOURCE_NAME} Language Server"),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Server ready!");
        let config = self.config_handle();
        match config.project_config_path() {
            Some(path) => info!("Project config: {}", path.display()),
            None => info!("Project config: <not set>"),
        }
        info!(
            "Global config: {}",
            config.global_config_path().unwrap_or_default().display()
        );
    }

    async fn shutdown(&self) -> RpcResult<()> {
        info!("Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!(
            "Opened document: uri {:?}, language: {}, version: {}",
            params.text_document.uri,
            params.text_document.language_id,
            params.text_document.version
        );
        self.document_cache.insert(&params.text_document);
        if self.should_spellcheck_while_typing() {
            self.spell_check(&params.text_document.uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_cache.remove(&params.text_document.uri);
        // Clear diagnostics when a file is closed.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        debug!("Saved document: {}", params.text_document.uri);
        if let Some(text) = params.text {
            self.document_cache.update(&params.text_document.uri, &text);
        }
        self.spell_check(&params.text_document.uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!(
            "Changed document: uri={}, version={}",
            params.text_document.uri, params.text_document.version
        );
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.first() {
            self.document_cache.update(&uri, &change.text);
            if self.should_spellcheck_while_typing() {
                self.spell_check(&uri).await;
            }
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> RpcResult<Option<CodeActionResponse>> {
        let mut actions: Vec<CodeActionOrCommand> = vec![];
        let doc = match self.document_cache.get(params.text_document.uri.as_ref()) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut has_codebook_diagnostic = false;
        for diag in params.context.diagnostics {
            // Only process our own diagnostics
            if diag.source.as_deref() != Some(SOURCE_NAME) {
                continue;
            }
            has_codebook_diagnostic = true;
            let line = doc
                .text
                .lines()
                .nth(diag.range.start.line as usize)
                .unwrap_or_default();
            let start_char = diag.range.start.character as usize;
            let end_char = diag.range.end.character as usize;
            let word = get_word_from_string(start_char, end_char, line);
            // info!("Word to suggest: {}", word);
            if word.is_empty() || word.contains(" ") {
                continue;
            }
            let cb = self.codebook_handle();
            let inner_word = word.clone();
            let suggestions = task::spawn_blocking(move || cb.get_suggestions(&inner_word)).await;

            let suggestions = match suggestions {
                Ok(suggestions) => suggestions,
                Err(e) => {
                    error!(
                        "Error getting suggestions for word '{}' in file '{}'\n Error: {}",
                        word,
                        doc.uri.path(),
                        e
                    );
                    continue;
                }
            };

            if suggestions.is_none() {
                continue;
            }

            suggestions.unwrap().iter().for_each(|suggestion| {
                actions.push(CodeActionOrCommand::CodeAction(self.make_suggestion(
                    suggestion,
                    &diag.range,
                    &params.text_document.uri,
                )));
            });
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Add '{word}' to dictionary"),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: format!("Add '{word}' to dictionary"),
                    command: CodebookCommand::AddWord.into(),
                    arguments: Some(vec![word.to_string().into()]),
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Add '{word}' to global dictionary"),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: format!("Add '{word}' to global dictionary"),
                    command: CodebookCommand::AddWordGlobal.into(),
                    arguments: Some(vec![word.to_string().into()]),
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
        }
        if has_codebook_diagnostic {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Add current file to ignore list".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: None,
                command: Some(Command {
                    title: "Add current file to ignore list".to_string(),
                    command: CodebookCommand::IgnoreFile.into(),
                    arguments: Some(vec![params.text_document.uri.to_string().into()]),
                }),
                is_preferred: None,
                disabled: None,
                data: None,
            }));
        }
        match actions.is_empty() {
            true => Ok(None),
            false => Ok(Some(actions)),
        }
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> RpcResult<Option<Value>> {
        match CodebookCommand::from(params.command.as_str()) {
            CodebookCommand::AddWord => {
                let config = self.config_handle();
                let words = params
                    .arguments
                    .iter()
                    .filter_map(|arg| arg.as_str().map(|s| s.to_string()));
                info!(
                    "Adding words to dictionary {}",
                    words.clone().collect::<Vec<String>>().join(", ")
                );
                let updated = self.add_words(config.as_ref(), words);
                if updated {
                    let _ = config.save();
                    self.recheck_all().await;
                }
                Ok(None)
            }
            CodebookCommand::AddWordGlobal => {
                let config = self.config_handle();
                let words = params
                    .arguments
                    .iter()
                    .filter_map(|arg| arg.as_str().map(|s| s.to_string()));
                let updated = self.add_words_global(config.as_ref(), words);
                if updated {
                    let _ = config.save_global();
                    self.recheck_all().await;
                }
                Ok(None)
            }
            CodebookCommand::IgnoreFile => {
                let Some(file_uri) = params
                    .arguments
                    .first()
                    .and_then(|arg| arg.as_str())
                else {
                    error!("IgnoreFile command missing or invalid file URI argument");
                    return Ok(None);
                };
                let config = self.config_handle();
                let updated = self.add_ignore_file(config.as_ref(), file_uri);
                if updated {
                    let _ = config.save();
                    self.recheck_all().await;
                }
                Ok(None)
            }
            CodebookCommand::Unknown => Ok(None),
        }
    }
}

impl Backend {
    pub fn new(client: Client, workspace_dir: &Path) -> Self {
        let workspace_dir_canonical = workspace_dir.canonicalize().ok();
        Self {
            client,
            workspace_dir: workspace_dir.to_path_buf(),
            workspace_dir_canonical,
            codebook: OnceLock::new(),
            config: OnceLock::new(),
            document_cache: TextDocumentCache::default(),
            initialize_options: RwLock::new(Arc::new(ClientInitializationOptions::default())),
        }
    }

    fn config_handle(&self) -> Arc<CodebookConfigFile> {
        self.config
            .get_or_init(|| {
                Arc::new(
                    CodebookConfigFile::load_with_global_config(
                        Some(self.workspace_dir.as_path()),
                        self.initialize_options
                            .read()
                            .unwrap()
                            .global_config_path
                            .clone(),
                    )
                    .expect("Unable to make config: {e}"),
                )
            })
            .clone()
    }

    fn codebook_handle(&self) -> Arc<Codebook> {
        self.codebook
            .get_or_init(|| {
                Arc::new(Codebook::new(self.config_handle()).expect("Unable to make codebook: {e}"))
            })
            .clone()
    }

    fn should_spellcheck_while_typing(&self) -> bool {
        self.initialize_options.read().unwrap().check_while_typing
    }

    fn make_diagnostic(&self, word: &str, start_pos: &Pos, end_pos: &Pos) -> Diagnostic {
        let message = format!("Possible spelling issue '{word}'.");
        Diagnostic {
            range: Range {
                start: Position {
                    line: start_pos.line as u32,
                    character: start_pos.col as u32,
                },
                end: Position {
                    line: end_pos.line as u32,
                    character: end_pos.col as u32,
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

    fn add_words(&self, config: &CodebookConfigFile, words: impl Iterator<Item = String>) -> bool {
        let mut should_save = false;
        for word in words {
            match config.add_word(&word) {
                Ok(true) => {
                    should_save = true;
                }
                Ok(false) => {
                    info!("Word '{word}' already exists in dictionary.");
                }
                Err(e) => {
                    error!("Failed to add word: {e}");
                }
            }
        }
        should_save
    }

    fn add_words_global(
        &self,
        config: &CodebookConfigFile,
        words: impl Iterator<Item = String>,
    ) -> bool {
        let mut should_save = false;
        for word in words {
            match config.add_word_global(&word) {
                Ok(true) => {
                    should_save = true;
                }
                Ok(false) => {
                    info!("Word '{word}' already exists in global dictionary.");
                }
                Err(e) => {
                    error!("Failed to add word: {e}");
                }
            }
        }
        should_save
    }

    fn get_relative_path(&self, uri: &str) -> Option<String> {
        let parsed_uri = match Url::parse(uri) {
            Ok(u) => u,
            Err(e) => {
                error!("Failed to parse URI '{uri}': {e}");
                return None;
            }
        };
        let file_path = parsed_uri.to_file_path().unwrap_or_default();
        Some(compute_relative_path(
            &self.workspace_dir,
            self.workspace_dir_canonical.as_deref(),
            &file_path,
        ))
    }

    fn add_ignore_file(&self, config: &CodebookConfigFile, file_uri: &str) -> bool {
        let Some(relative_path) = self.get_relative_path(file_uri) else {
            return false;
        };
        match config.add_ignore(&relative_path) {
            Ok(true) => true,
            Ok(false) => {
                info!("File {file_uri} already exists in the ignored files.");
                false
            }
            Err(e) => {
                error!("Failed to add ignore file: {e}");
                false
            }
        }
    }

    fn make_suggestion(&self, suggestion: &str, range: &Range, uri: &Url) -> CodeAction {
        let title = format!("Replace with '{suggestion}'");
        let mut map = HashMap::new();
        map.insert(
            uri.clone(),
            vec![TextEdit {
                range: *range,
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
        debug!("Rechecking documents: {urls:?}");
        for url in urls {
            self.publish_spellcheck_diagnostics(&url).await;
        }
    }

    async fn spell_check(&self, uri: &Url) {
        let config = self.config_handle();
        let did_reload = match config.reload() {
            Ok(did_reload) => did_reload,
            Err(e) => {
                error!("Failed to reload config: {e}");
                false
            }
        };

        if did_reload {
            debug!("Config reloaded, rechecking all files.");
            self.recheck_all().await;
        } else {
            debug!("Checking file: {uri:?}");
            self.publish_spellcheck_diagnostics(uri).await;
        }
    }

    /// Helper method to publish diagnostics for spell-checking.
    async fn publish_spellcheck_diagnostics(&self, uri: &Url) {
        let doc = match self.document_cache.get(uri.as_ref()) {
            Some(doc) => doc,
            None => return,
        };
        // Convert the file URI to a local file path.
        let file_path = doc.uri.to_file_path().unwrap_or_default();
        debug!("Spell-checking file: {file_path:?}");

        // Compute relative path for ignore pattern matching
        let relative_path = compute_relative_path(
            &self.workspace_dir,
            self.workspace_dir_canonical.as_deref(),
            &file_path,
        );

        // Convert utf8 byte offsets to utf16
        let offsets = StringOffsets::<AllConfig>::new(&doc.text);

        // Perform spell-check.
        let lang = doc.language_id.as_deref();
        let lang_type = lang.and_then(|lang| LanguageType::from_str(lang).ok());
        debug!("Document identified as type {lang_type:?} from {lang:?}");
        let cb = self.codebook_handle();
        let spell_results = task::spawn_blocking(move || {
            cb.spell_check(&doc.text, lang_type, Some(&relative_path))
        })
        .await;

        let spell_results = match spell_results {
            Ok(results) => results,
            Err(err) => {
                error!("Spell-checking failed for file '{file_path:?}' \n Error: {err}");
                return;
            }
        };

        // Convert the results to LSP diagnostics.
        let diagnostics: Vec<Diagnostic> = spell_results
            .into_iter()
            .flat_map(|res| {
                // For each misspelling, create a diagnostic for each location.
                let mut new_locations = vec![];
                for loc in &res.locations {
                    let start_pos = offsets.utf8_to_utf16_pos(loc.start_byte);
                    let end_pos = offsets.utf8_to_utf16_pos(loc.end_byte);
                    let diagnostic = self.make_diagnostic(&res.word, &start_pos, &end_pos);
                    new_locations.push(diagnostic);
                }
                new_locations
            })
            .collect();

        // debug!("Diagnostics: {:?}", diagnostics);
        // Send the diagnostics to the client.
        self.client
            .publish_diagnostics(doc.uri, diagnostics, None)
            .await;
        // debug!("Published diagnostics for: {:?}", file_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compute_relative_path_within_workspace() {
        let workspace = tempdir().unwrap();
        let workspace_path = workspace.path();

        // Create a file inside the workspace
        let subdir = workspace_path.join("src");
        fs::create_dir_all(&subdir).unwrap();
        let file_path = subdir.join("test.rs");
        fs::write(&file_path, "test").unwrap();

        let result = compute_relative_path(workspace_path, None, &file_path);
        assert_eq!(result, "src/test.rs");
    }

    #[test]
    fn test_compute_relative_path_with_cached_canonical() {
        let workspace = tempdir().unwrap();
        let workspace_path = workspace.path();
        let workspace_canonical = workspace_path.canonicalize().unwrap();

        // Create a file inside the workspace
        let subdir = workspace_path.join("src");
        fs::create_dir_all(&subdir).unwrap();
        let file_path = subdir.join("test.rs");
        fs::write(&file_path, "test").unwrap();

        // Using cached canonical path should produce the same result
        let result = compute_relative_path(workspace_path, Some(&workspace_canonical), &file_path);
        assert_eq!(result, "src/test.rs");
    }

    #[test]
    fn test_compute_relative_path_outside_workspace() {
        let workspace = tempdir().unwrap();
        let other_dir = tempdir().unwrap();

        // Create a file outside the workspace
        let file_path = other_dir.path().join("outside.rs");
        fs::write(&file_path, "test").unwrap();

        let result = compute_relative_path(workspace.path(), None, &file_path);
        // Should return the original path since it's outside workspace
        assert!(result.contains("outside.rs"));
    }

    #[test]
    fn test_compute_relative_path_nonexistent_file() {
        let workspace = tempdir().unwrap();
        let file_path = workspace.path().join("nonexistent.rs");

        let result = compute_relative_path(workspace.path(), None, &file_path);
        // Should return the original path since file doesn't exist
        assert!(result.contains("nonexistent.rs"));
    }

    #[test]
    fn test_compute_relative_path_nested_directory() {
        let workspace = tempdir().unwrap();
        let workspace_path = workspace.path();

        // Create a deeply nested file
        let nested_dir = workspace_path.join("src").join("components").join("ui");
        fs::create_dir_all(&nested_dir).unwrap();
        let file_path = nested_dir.join("button.rs");
        fs::write(&file_path, "test").unwrap();

        let result = compute_relative_path(workspace_path, None, &file_path);
        assert_eq!(result, "src/components/ui/button.rs");
    }
}
