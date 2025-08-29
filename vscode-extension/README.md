# Codebook VS Code Extension

A VS Code extension for [Codebook](https://github.com/blopker/codebook) - a code-aware spell checker that understands your code structure.

## Features

- **Code-aware spell checking**: Intelligently checks spelling in code comments, strings, and documentation while ignoring code elements
- **Language Server Protocol**: Full LSP support for real-time spell checking
- **Multi-language support**: Works with JavaScript, TypeScript, Python, Rust, Go, and many more languages
- **Custom dictionaries**: Add project-specific and user-specific words
- **Hierarchical configuration**: Global, user, and project-level configuration support

## Requirements

### Install the Language Server

The extension requires the `codebook-lsp` language server to be installed on your system.

#### Option 1: Install via Cargo (Recommended)
```bash
cargo install codebook-lsp
```

#### Option 2: Download Pre-built Binary
Download the latest release from [GitHub Releases](https://github.com/blopker/codebook/releases) and add it to your PATH.

## Extension Settings

This extension contributes the following settings:

* `codebook.enable`: Enable/disable the spell checker
* `codebook.serverPath`: Path to the codebook-lsp executable (default: `codebook-lsp`)
* `codebook.logLevel`: Log level for the language server (`error`, `warn`, `info`, `debug`, `trace`)
* `codebook.minWordLength`: Minimum word length to check (default: 3)
* `codebook.dictionaries`: Additional dictionaries to use
* `codebook.words`: Custom words to add to the dictionary
* `codebook.flagWords`: Words to always flag as misspelled
* `codebook.ignorePaths`: Paths to ignore during spell checking
* `codebook.ignorePatterns`: Regex patterns to ignore
* `codebook.useGlobalConfig`: Use global configuration in addition to project configuration

## Commands

The extension provides the following commands:

* `Codebook: Add to Workspace Dictionary` - Add the current word to the workspace dictionary
* `Codebook: Add to User Dictionary` - Add the current word to your user dictionary
* `Codebook: Restart Language Server` - Restart the Codebook language server
* `Codebook: Show Output Channel` - Show the Codebook output channel for debugging

## Configuration Files

Codebook uses TOML configuration files:

### Project Configuration
Create a `codebook.toml` file in your project root:

```toml
# Custom words for this project
words = ["myapp", "api", "webpack"]

# Words to flag as errors
flag_words = ["todo", "fixme"]

# Paths to ignore
ignore_paths = ["node_modules", "dist", "build"]

# Regex patterns to ignore
ignore_patterns = [
    "\\bhttps?://[^\\s]+",  # URLs
    "\\b[A-Z0-9]{8}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{12}\\b"  # UUIDs
]

# Minimum word length
min_word_length = 3
```

### User Configuration
Create a configuration file at `~/.config/codebook/codebook.toml` for user-wide settings.

## Quick Actions

When Codebook detects a spelling error, you can:

1. **Quick Fix**: Use VS Code's Quick Fix feature (Ctrl+. or Cmd+.) to:
   - Add the word to your workspace dictionary
   - Add the word to your user dictionary
   - See spelling suggestions

2. **Code Actions**: Right-click on a misspelled word to access Codebook actions

## Troubleshooting

### Language Server Not Found

If you get an error about the language server not being found:

1. Ensure `codebook-lsp` is installed: `cargo install codebook-lsp`
2. Check that it's in your PATH: `which codebook-lsp` (or `where codebook-lsp` on Windows)
3. If installed in a custom location, set the `codebook.serverPath` setting to the full path

### No Spell Checking

If spell checking isn't working:

1. Check the Output Channel: Run "Codebook: Show Output Channel" command
2. Verify the language server is running: Check the status bar
3. Ensure the file type is supported
4. Check your configuration files for syntax errors

### Performance Issues

For large projects, you can improve performance by:

1. Adding more paths to `ignore_paths` in your configuration
2. Using `ignore_patterns` to exclude specific text patterns
3. Increasing the `min_word_length` setting

## Development

To build and test the extension locally:

```bash
# Clone the repository
git clone https://github.com/blopker/codebook.git
cd codebook/vscode-extension

# Install dependencies
npm install

# Compile the extension
npm run compile

# Package the extension
npm run package
```

To test the extension:
1. Open the extension folder in VS Code
2. Press F5 to launch a new VS Code window with the extension loaded
3. Open a file to test spell checking

## Contributing

Contributions are welcome! Please see the main [Codebook repository](https://github.com/blopker/codebook) for contribution guidelines.

## License

MIT - See the [LICENSE](https://github.com/blopker/codebook/blob/main/LICENSE) file for details.