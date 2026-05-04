# Changelog

## Unreleased

- Fix `command 'codebook.addWord' already exists` crash when opening a multi-root workspace (#250). The extension now starts a single language server for the whole window. Note: in multi-root workspaces only the first folder's `codebook.toml` is honored; full per-folder project config support is tracked separately.

## 0.0.1

- Initial release
- Code-aware spell checking powered by Tree-sitter via the `codebook-lsp` language server
- Automatic binary download and management for macOS, Linux, and Windows
- Support for 35+ programming languages
- Hierarchical configuration with per-project and global `codebook.toml` files
- Code actions: add to project dictionary, add to global dictionary, ignore
- Multi-root workspace support
