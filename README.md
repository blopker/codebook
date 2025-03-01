<br />
<div align="center">
  <a href="https://github.com/blopker/codebook"> <img
  src="assets/codebook-nt.webp" alt="Logo" width="200" > </a> <h3
  align="center">CODEBOOK</h3> <p align="center"> An unholy spellchecker for
  code. <br /> <br /> <!-- <a
  href="https://github.com/blopker/codebook/releases/latest/">Download</a> -->
  <br /> <br /> <a href="https://github.com/blopker/codebook/issues">Report
  Bug</a> · <a href="https://github.com/blopker/codebook/issues">Request
  Feature</a> </p>
</div>

## Usage

<img
  src="assets/example.png" alt="Logo" width="400" >

No setup needed. Code book will automatically detect the language you are editing and mark issues for you. Note, Codebook will only mark issues that you control, where they are initially defined.

## Install

Codebook is currently only available for the Zed editor. To install, go to the Extension tab in Zed and look for "Codebook".

## About

Codebook is a spellchecker for code. It binds together the venerable Tree Sitter and the fast spell checker [Spellbook](https://github.com/helix-editor/spellbook). Included is a Language Server for use in any editor. Everything is done in Rust to keep response times snappy and memory usage _low_.

## Status

Codebook is being developed, but the Zed extension is now live!

### Supported Languages

| Language | Status |
| --- | --- |
| Markdown | ✅ |
| Plain Text | ✅ |
| C | ✅ |
| HTML | ⚠️ |
| CSS | ⚠️ |
| Go | ⚠️ |
| JavaScript | ✅ |
| TypeScript | ✅ |
| Python | ✅ |
| Rust | ✅ |
| TOML | ✅ |

✅ = Good to go
⚠️ = Supported, but needs more testing

## Configuration

Codebook supports both global and project-specific configuration. Configuration files use the TOML format, with project settings overriding global ones.

### Global Configuration

The global configuration applies to all projects by default. Location depends on your operating system:

- **Linux/macOS**: `$XDG_CONFIG_HOME/codebook/codebook.toml` or `~/.config/codebook/codebook.toml`
- **Windows**: `%APPDATA%\codebook\codebook.toml`

### Project Configuration

Project-specific configuration is loaded from either `codebook.toml` or `.codebook.toml` in the project root. Codebook searches for this file starting from the current directory and moving up to parent directories.

### Configuration Options

```toml
# List of dictionaries to use for spell checking
# Default: ["en_us"]
# Other options include "en_gb"
dictionaries = ["en_us", "en_gb"]

# Custom allowlist of words to ignore (case-insensitive)
# Codebook will add words here when you select "Add to dictionary"
# Default: []
words = ["codebook", "rustc"]

# Words that should always be flagged as incorrect
# Default: []
flag_words = ["todo", "fixme"]

# List of glob patterns for paths to ignore when spell checking
# Default: []
ignore_paths = ["target/**/*", "**/*.json", ".git/**/*"]

# List of regex patterns to ignore when spell checking
# Useful for domain-specific strings or patterns
# Default: []
ignore_patterns = [
    "^[ATCG]+$",             # DNA sequences
    "\\d{3}-\\d{2}-\\d{4}"   # Social Security Number format
]

# Whether to use global configuration (project config only)
# Set to false to completely ignore global settings
# Default: true
use_global = true
```

### Configuration Precedence

1. Project configuration overrides global configuration
2. If `use_global = false` in project config, global settings are ignored entirely
3. If no project config exists, global config is used
4. If neither exists, default settings are used

### Working with Configurations

- Words added with "Add to dictionary" are stored in the project configuration
- Project settings are saved automatically when words are added
- Configuration files are automatically reloaded when they change

## Goals

Spellchecking is complicated and opinions about how it should be done, especially with code, differs. This section is about the trade offs that steer decisions.

### Privacy

No remote calls for spellchecking or analytics. Once dictionaries are cached, Codebook needs to be usable offline. Codebook will never send the contents of files to a remote server.

### Don't be annoying

Codebook should have high signal and low noise. It should only highlight words that users have control over. For example, a misspelled word in an imported function should not be highlighted as the user can't do anything about it.

### Efficient

All features will be weighed against their impact on CPU and memory. Codebook should be fast enough to spellcheck on every keystroke on even low-end hardware.

## Features

### Code-aware spell checking

Codebook will only check the parts of your code where a normal linter wouldn't. Comments, string literals and variable definitions for example. Codebook knows how to split camel case and snake case variables, and makes suggestions in the original case.

### Language Server

Codebook comes with a language server. Originally developed for the Zed editor, this language server can be integrated into any editor that supports the language server protocol.

### Dictionary Management

Codebook comes with a dictionary manager, which will automatically download and cache dictionaries for a large number of written languages.

### Hierarchical Configuration

Codebook uses a hierarchical configuration system with global (user-level) and project-specific settings, giving you flexibility to set defaults and override them as needed per project.

## Roadmap

- [X] Support more languages than US English
- [X] Support custom project dictionaries
- [X] Support per file extension dictionaries
- [X] Add code actions to correct spelling
- [X] Support hierarchical global and project configuration

## Running Tests

Run test with `make test` after cloning.

## Acknowledgments
- Harper: https://writewithharper.com/
- Harper Zed: https://github.com/Stef16Robbe/harper_zed
- Spellbook: https://github.com/helix-editor/spellbook
- cSpell for VSCode: https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker
- Vale: https://github.com/errata-ai/vale-ls
- TreeSitter Visualizer: https://intmainreturn0.com/ts-visualizer/
- common-words: https://github.com/anvaka/common-words
- Hunspell dictionaries in UTF-8: https://github.com/wooorm/dictionaries

## Release

To update the Language server:

1. Run `make release-lsp`
1. Follow instructions
1. Go to GitHub Releases
1. Un-mark "prerelease" and publish

To update the Zed Extension:

1. Go to blopker/codebook-zed
1. Update the version in extension.toml
1. Run `git submodule update --remote --merge extensions/codebook` in zed/extensions
1. Make a PR to zed/extensions with the updated submodule
