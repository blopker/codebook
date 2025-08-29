# Changelog

All notable changes to the Codebook VS Code extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-01-XX

### Added
- Initial release of Codebook VS Code extension
- Language Server Protocol (LSP) client implementation for codebook-lsp
- Code-aware spell checking for multiple programming languages
- Support for custom dictionaries and word lists
- Commands to add words to workspace and user dictionaries
- Configuration options for:
  - Server path customization
  - Log level control
  - Minimum word length
  - Custom dictionaries
  - Ignore paths and patterns
  - Flag words
- Quick fix code actions for misspelled words
- Auto-detection of codebook-lsp in PATH and cargo installation directory
- File watchers for configuration file changes (codebook.toml, .codebookrc, .codebook.toml)
- Output channel for debugging and diagnostics
- Command to restart the language server
- Support for both file and untitled documents

### Features
- **Multi-language support**: JavaScript, TypeScript, Python, Rust, Go, Java, C/C++, C#, Ruby, PHP, Swift, Kotlin, Scala, and many more
- **Hierarchical configuration**: Global, user, and project-level settings
- **Real-time spell checking**: Instant feedback as you type
- **Smart code analysis**: Understands code structure and only checks appropriate text
- **Custom dictionaries**: Add project-specific technical terms
- **Pattern ignoring**: Skip URLs, UUIDs, and other patterns using regex

### Technical Details
- Built with TypeScript and vscode-languageclient
- Requires codebook-lsp server (installable via cargo)
- Uses stdio transport for LSP communication
- Supports VS Code 1.85.0 and later

## Future Releases

### Planned Features
- Inline spell check suggestions
- Batch operations for adding multiple words
- Dictionary management UI
- Performance optimizations for large files
- Support for workspace-specific language settings
- Integration with VS Code's problems panel
- Custom severity levels for different word categories