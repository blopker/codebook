use zed_extension_api::settings::LspSettings;
use zed_extension_api::{
    self as zed,
    lsp::{Completion, Symbol},
    CodeLabel, Command, LanguageServerId, Result, Worktree,
};

struct SpellcheckExtension {
    // ... stat
}

impl zed::Extension for SpellcheckExtension {
    fn new() -> Self {
        Self {}
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        eprintln!("yooooooooooooooooooooooo");
        println!("yooooooooooooooooooooooo2");
        eprintln!("language_server_command: {:?}", settings);
        Ok(Command {
            command: "codebook-lsp".to_string(),
            args: vec![],
            env: vec![],
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        _completion: Completion,
    ) -> Option<CodeLabel> {
        eprintln!("label_for_completion: {:?}", _completion);
        None
    }
    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        _symbol: Symbol,
    ) -> Option<CodeLabel> {
        eprintln!("label_for_symbol: {:?}", _symbol);
        None
    }
}

zed::register_extension!(SpellcheckExtension);
