# Codebook VS Code Extension

This extension wires the Codebook language server into VS Code so code-specific
spell check diagnostics and quick fixes appear automatically.

## Features

- Automatically launches `codebook-lsp` for supported languages.
- Downloads, caches, and updates the Codebook language server without requiring any manual installation.
- Supports custom binary locations and optional pre-release builds via
  `codebook.*` settings.

## Development

```
cd editors/vscode
bun install   # or npm install / pnpm install
bun run build # or npm run build
bun run package # builds dist/ and emits a .vsix via vsce
```

The emitted JavaScript lives in `dist/` and can be loaded into VS Code via the
`Extension Tests / Run Extension` launch configuration or by using the bundled
`vsce` CLI via `bun run package`.

Set `codebook.logLevel` to `debug` to see verbose logs from the language server
inside the `Codebook` output channel.
