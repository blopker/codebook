use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;
use std::sync::{Arc, RwLock};

use codebook::parser::get_word_from_string;
use codebook::queries::LanguageType;
use string_offsets::AllConfig;
use string_offsets::Pos;
use string_offsets::StringOffsets;

use log::LevelFilter;
use log::error;
use serde::Deserialize;
use serde_json::Value;
use tokio::task;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use codebook::Codebook;
use codebook_config::{CodebookConfig, CodebookConfigFile};
use log::{debug, info};

use crate::file_cache::TextDocumentCache;
use crate::lsp_logger;

const SOURCE_NAME: &str = "Codebook";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientInitializationOptions {
    #[serde(default)]
    log_level: Option<String>,
    #[serde(default)]
    global_config_path: Option<Option<String>>,
}

impl ClientInitializationOptions {
    fn from_value(value: Option<Value>) -> Self {
        value
            .and_then(|options| {
                serde_json::from_value(options)
                    .map_err(|err| {
                        error!("Failed to parse initialization options: {err}");
                        err
                    })
                    .ok()
            })
            .unwrap_or_default()
    }

    fn log_level_filter(&self) -> LevelFilter {
        match self
            .log_level
            .as_deref()
            .map(|level| level.to_ascii_lowercase())
        {
            Some(level) if level == "trace" => LevelFilter::Trace,
            Some(level) if level == "debug" => LevelFilter::Debug,
            Some(level) if level == "warn" => LevelFilter::Warn,
            Some(level) if level == "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        }
    }

    fn global_config_override(&self) -> Option<Option<PathBuf>> {
        self.global_config_path.as_ref().map(|maybe_path| {
            maybe_path.as_ref().and_then(|path| {
                let trimmed = path.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(trimmed))
                }
            })
        })
    }
}

pub struct Backend {
    pub client: Client,
    // Wrap every call to codebook in spawn_blocking, it's not async
    workspace_dir: PathBuf,
    global_config_override: RwLock<Option<PathBuf>>,
    pub codebook: RwLock<Arc<Codebook>>,
    pub config: RwLock<Arc<CodebookConfigFile>>,
    pub document_cache: TextDocumentCache,
}

enum CodebookCommand {
    AddWord,
    AddWordGlobal,
    Unknown,
}

impl From<&str> for CodebookCommand {
    fn from(command: &str) -> Self {
        match command {
            "codebook.addWord" => CodebookCommand::AddWord,
            "codebook.addWordGlobal" => CodebookCommand::AddWordGlobal,
            _ => CodebookCommand::Unknown,
        }
    }
}

impl From<CodebookCommand> for String {
    fn from(command: CodebookCommand) -> Self {
        match command {
            CodebookCommand::AddWord => "codebook.addWord".to_string(),
            CodebookCommand::AddWordGlobal => "codebook.addWordGlobal".to_string(),
            CodebookCommand::Unknown => "codebook.unknown".to_string(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> RpcResult<InitializeResult> {
        // info!("Capabilities: {:?}", params.capabilities);
        let client_options =
            ClientInitializationOptions::from_value(params.initialization_options.clone());
        let log_level = client_options.log_level_filter();
        let global_override = client_options.global_config_override();

        // Attach the LSP client to the logger and flush buffered logs
        lsp_logger::LspLogger::attach_client(self.client.clone(), log_level);
        info!(
            "LSP logger attached to client with log level: {}",
            log_level
        );
        if let Some(global_override) = global_override {
            if self.apply_global_config_override(global_override.clone()) {
                match &global_override {
                    Some(path) => {
                        info!(
                            "Using client-supplied global config path: {}",
                            path.display()
                        );
                    }
                    None => info!("Client reset global config override to default location"),
                }
            }
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF16),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        CodebookCommand::AddWord.into(),
                        CodebookCommand::AddWordGlobal.into(),
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
        debug!("Saved document: {}", params.text_document.uri);
        if let Some(text) = params.text {
            self.document_cache.update(&params.text_document.uri, &text);
            self.spell_check(&params.text_document.uri).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        debug!(
            "Changed document: uri={}, version={}",
            params.text_document.uri, params.text_document.version
        );
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.first() {
            self.document_cache.update(&uri, &change.text);
            self.spell_check(&uri).await;
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> RpcResult<Option<CodeActionResponse>> {
        let mut actions: Vec<CodeActionOrCommand> = vec![];
        let doc = match self.document_cache.get(params.text_document.uri.as_ref()) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        for diag in params.context.diagnostics {
            // Only process our own diagnostics
            if diag.source.as_deref() != Some(SOURCE_NAME) {
                continue;
            }
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
            CodebookCommand::Unknown => Ok(None),
        }
    }
}

impl Backend {
    pub fn new(client: Client, workspace_dir: &Path) -> Self {
        let initial_override: Option<PathBuf> = None;
        let (config_arc, codebook) =
            Self::build_configuration(workspace_dir, initial_override.as_deref())
                .expect("Unable to initialize Codebook configuration");

        Self {
            client,
            workspace_dir: workspace_dir.to_path_buf(),
            global_config_override: RwLock::new(initial_override),
            codebook: RwLock::new(codebook),
            config: RwLock::new(config_arc),
            document_cache: TextDocumentCache::default(),
        }
    }

    fn build_configuration(
        workspace_dir: &Path,
        global_config_override: Option<&Path>,
    ) -> Result<(Arc<CodebookConfigFile>, Arc<Codebook>), String> {
        let config = CodebookConfigFile::load_with_global_config(
            Some(workspace_dir),
            global_config_override,
        )
        .map_err(|e| format!("Unable to make config: {e}"))?;
        let config_arc: Arc<CodebookConfigFile> = Arc::new(config);
        let cb_config = Arc::clone(&config_arc);
        let codebook =
            Codebook::new(cb_config).map_err(|e| format!("Unable to make codebook: {e}"))?;
        Ok((config_arc, Arc::new(codebook)))
    }

    fn apply_global_config_override(&self, new_override: Option<PathBuf>) -> bool {
        {
            let guard = self.global_config_override.read().unwrap();
            if *guard == new_override {
                return false;
            }
        }

        match Self::build_configuration(&self.workspace_dir, new_override.as_deref()) {
            Ok((config_arc, codebook)) => {
                {
                    let mut cfg = self.config.write().unwrap();
                    *cfg = Arc::clone(&config_arc);
                }
                {
                    let mut cb = self.codebook.write().unwrap();
                    *cb = codebook;
                }
                {
                    let mut guard = self.global_config_override.write().unwrap();
                    *guard = new_override;
                }
                true
            }
            Err(err) => {
                error!("Failed to apply global config override: {err}");
                false
            }
        }
    }

    fn config_handle(&self) -> Arc<CodebookConfigFile> {
        self.config.read().unwrap().clone()
    }

    fn codebook_handle(&self) -> Arc<Codebook> {
        self.codebook.read().unwrap().clone()
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

        // Convert utf8 byte offsets to utf16
        let offsets = StringOffsets::<AllConfig>::new(&doc.text);

        // Perform spell-check.
        let lang = doc.language_id.as_deref();
        let lang_type = lang.and_then(|lang| LanguageType::from_str(lang).ok());
        debug!("Document identified as type {lang_type:?} from {lang:?}");
        let cb = self.codebook_handle();
        let fp = file_path.clone();
        let spell_results = task::spawn_blocking(move || {
            cb.spell_check(&doc.text, lang_type, Some(fp.to_str().unwrap_or_default()))
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
