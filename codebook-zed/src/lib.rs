use zed_extension_api::settings::LspSettings;
use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

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
}

zed::register_extension!(SpellcheckExtension);
