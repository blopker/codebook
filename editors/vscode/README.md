# Codebook VS Code Extension

This extension wires the Codebook language server into VS Code so code-specific
spell check diagnostics and quick fixes appear automatically.

## Features

- Automatically launches `codebook-lsp` for C, C++, CSS, Elixir, Go, HTML,
  Haskell, Java, JavaScript, LaTeX, Lua, Markdown, PHP, plain text, Python,
  Ruby, Rust, TOML, TypeScript, Typst, Zig, and C# files.
- Downloads, caches, and updates the Codebook language server without requiring
  any manual installation.
- Supports custom binary locations and optional pre-release builds via
  `codebook.*` settings.

## Development

```
cd editors/vscode
bun install   # or npm install / pnpm install
bun run build # or npm run build
```

The emitted JavaScript lives in `dist/` and can be loaded into VS Code via the
`Extension Tests / Run Extension` launch configuration or by using
`vsce package`.

Set `codebook.logLevel` to `debug` to see verbose logs from the language server
inside the `Codebook` output channel.
