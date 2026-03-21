<div align="center">
  <img src="https://raw.githubusercontent.com/blopker/codebook/main/assets/codebook-nt.webp" width="120">
  <h1>Codebook Spell Checker</h1>
  <p>A fast, code-aware spell checker — built for code, not prose.</p>

  [![Version](https://img.shields.io/visual-studio-marketplace/v/blopker.codebook-vscode?color=blue&label=VS%20Code)](https://marketplace.visualstudio.com/items?itemName=blopker.codebook-vscode)
  [![Installs](https://img.shields.io/visual-studio-marketplace/i/blopker.codebook-vscode)](https://marketplace.visualstudio.com/items?itemName=blopker.codebook-vscode)
  [![Rating](https://img.shields.io/visual-studio-marketplace/r/blopker.codebook-vscode)](https://marketplace.visualstudio.com/items?itemName=blopker.codebook-vscode)
  [![License](https://img.shields.io/github/license/blopker/codebook)](LICENSE)

  <br/>
  <a href="https://github.com/blopker/codebook/issues">Report a Bug</a> ·
  <a href="https://github.com/blopker/codebook/issues">Request a Feature</a> ·
  <a href="https://blopker.com/writing/09-survey-of-the-current-state-of-code-spell-checking/">Why Codebook?</a>
</div>

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Extension Settings](#extension-settings)
- [Configuration File](#configuration-file)
- [Supported Languages](#supported-languages)
- [Contributing](#contributing)

---

## Features

![Codebook in action](https://github.com/blopker/codebook/blob/main/assets/example.png?raw=true)

Codebook combines [Tree Sitter](https://tree-sitter.github.io) and [Spellbook](https://github.com/helix-editor/spellbook) into a Language Server written in Rust. It's designed to be fast and light on memory. If you need a traditional spell checker for prose, Codebook is probably not the right fit: it handles capitalization loosely by design and does not do grammar checking.

**Code-aware checking** - Codebook only looks at the parts of your code where spelling actually matters: variable and function definitions, comments, string literals, and documentation. It knows how to split camelCase and snake_case, and suggests fixes in the same casing style.

**Zero configuration** - install it and it works. No dictionaries to configure, no files to create.

**Definitions only, not usages** - if a typo comes from an imported function name, Codebook won't flag it since you can't change it anyway. It only marks words at the point where you defined them, so every warning is something you can actually fix.

**Private and offline** - everything runs locally. No file contents are ever sent anywhere. Once the dictionaries are downloaded, Codebook works without an internet connection.

**Hierarchical config** - set global defaults and override them per project. Works across workspaces.

---

## Requirements

- **VS Code** `1.80.0` or later
- **OS:** macOS (x86_64, aarch64), Linux (x86_64, aarch64), Windows (x86_64, arm64)
- No other dependencies. The `codebook-lsp` binary is downloaded and managed automatically on first activation.

---

## Installation

Install **Codebook Spell Checker** from the VS Code Marketplace:

1. Open VS Code.
2. Open the Extensions panel (`Ctrl+Shift+X` / `Cmd+Shift+X`).
3. Search for **`Codebook Spell Checker`**.
4. Click **Install**.

Or install from the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`):

```
ext install blopker.codebook-vscode
```

On first activation, Codebook downloads the `codebook-lsp` binary in the background. An internet connection is needed for this one-time step.

---

## Usage

Codebook activates automatically for all [supported languages](#supported-languages). No setup needed.

Spelling errors are highlighted with red squiggles in:
- Variable and function names (at the point of definition)
- Comments and documentation
- String literals

To fix an error, right-click on the highlighted word:

| Action | Description |
|--------|-------------|
| **Add to project dictionary** | Saves the word to your project's `codebook.toml` |
| **Add to global dictionary** | Saves the word to your global Codebook config |
| **Ignore** | Dismisses the warning for this session |

> Codebook only flags words at their *definition*. Use your language's rename/refactor tool to update all usages at once.

---

## Extension Settings

Settings can be configured in the Settings UI (`Ctrl+,`) or in `settings.json`.

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `codebook.binaryPath` | `string` | `""` | Path to a custom `codebook-lsp` binary. Leave empty to use the auto-managed one. |
| `codebook.enablePrerelease` | `boolean` | `false` | Use pre-release versions of `codebook-lsp`. |
| `codebook.logLevel` | `string` | `"info"` | Log verbosity: `debug`, `info`, `warn`, or `error`. |

**Example `settings.json`:**

```jsonc
{
  "codebook.binaryPath": "/usr/local/bin/codebook-lsp",
  "codebook.enablePrerelease": false,
  "codebook.logLevel": "info"
}
```

---

## Configuration File

Codebook supports per-project and global configuration via TOML files.

### File locations

| Scope | Path |
|-------|------|
| **Project** | `codebook.toml` or `.codebook.toml` in the project root |
| **Global (Linux/macOS)** | `~/.config/codebook/codebook.toml` |
| **Global (Windows)** | `%APPDATA%\codebook\codebook.toml` |

Project settings override global ones. Codebook searches up the directory tree to find the nearest project config.

> **Tip:** If you create or rename a config file manually, reload the window (`Ctrl+Shift+P` > "Developer: Reload Window") to pick up the change.

### All options

```toml
# Dictionaries to use. Default: ["en_us"]
# Available: en_us, en_gb, de, de_at, de_ch, nl_nl, es, fr, it,
#            pt_br, ru, sv, da, lv, vi_vn, pl, uk
dictionaries = ["en_us", "en_gb"]

# Words to always allow (case-insensitive). Codebook adds words here
# when you select "Add to project dictionary".
words = ["codebook", "rustc"]

# Words that should always be flagged as incorrect.
flag_words = ["todo", "fixme"]

# Only spell-check files matching these glob patterns (allowlist).
# Default: [] (check everything)
include_paths = ["src/**/*.rs", "lib/**/*.rs"]

# Skip files matching these glob patterns (blocklist).
# Takes precedence over include_paths.
ignore_paths = ["target/**/*", "**/*.json", ".git/**/*"]

# Regex patterns — tokens within matches are skipped.
# Use single quotes to avoid backslash escaping.
ignore_patterns = [
  '\b[ATCG]+\b',        # DNA sequences
  '\d{3}-\d{2}-\d{4}',  # Social Security Numbers
]

# Minimum word length to check. Default: 3
min_word_length = 3

# Set to false to ignore the global config for this project.
# Default: true
use_global = true
```

### Precedence

1. Project config overrides global config
2. If `use_global = false`, the global config is ignored entirely for that project
3. If no project config exists, the global config applies
4. If neither exists, defaults are used

### How words are saved

- **Add to dictionary** saves to the project config
- **Add to global dictionary** saves to the global config
- Changes are written automatically and reloaded without a restart

### User-defined regex patterns

Use `ignore_patterns` to skip tokens that match a custom regex. Some things to know:

**Built-in patterns** - these are already ignored by default:

| Pattern | Matches |
|---------|---------|
| `https?://[^\s]+` | URLs |
| `#[0-9a-fA-F]{3,8}` | Hex colors (`#deadbeef`, `#fff`) |
| `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` | Email addresses |
| `/[^\s]*` and `[A-Za-z]:\\[^\s]*` | Unix and Windows file paths |
| `[0-9a-fA-F]{8}-...-[0-9a-fA-F]{12}` | UUIDs |
| `[A-Za-z0-9+/]{20,}={0,2}` | Base64 strings (20+ chars) |
| `\b[0-9a-fA-F]{7,40}\b` | Git commit hashes |
| `\[([^\]]+)\]\(([^)]+)\)` | Markdown link targets |

**How matching works:**

- Patterns run against the full source text; any token inside a match is skipped
- Multiline mode is on: `^` and `$` match line boundaries
- Use single-quoted TOML strings to avoid double-escaping backslashes

```toml
ignore_patterns = [
  '\b[ATCG]+\b',   # DNA sequences
  '^vim\..*',      # Lines starting with vim.
  '^\s*//.*',      # Full-line // comments
]
```

---

## Supported Languages

Codebook supports 35+ programming languages. See the [full supported language list](https://github.com/blopker/codebook#supported-languages) in the main repository for current status.

---

## Contributing

Contributions are welcome. To get started:

1. Fork the repository on GitHub.
2. Clone your fork: `git clone https://github.com/YOUR_USER/codebook`
3. Install dependencies: `npm install` (extension) and `cargo build` (LSP server)
4. Open the project in VS Code and press `F5` to launch the Extension Development Host.
5. Submit a pull request with a description of your change.

If you use a language marked ⚠️, the most useful thing you can do is test it and report what you find.
