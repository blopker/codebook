<br />
<div align="center">
  <a href="https://github.com/blopker/codebook">
    <img src="assets/codebook-nt.webp" alt="Logo" width="200" >
  </a>
  <h3 align="center">CODEBOOK</h3>
  <p align="center">
      An unholy spellchecker for code.
    <br />
    <br />
    <!-- <a href="https://github.com/blopker/codebook/releases/latest/">Download</a> -->
    <br />
    <br />
    <a href="https://github.com/blopker/codebook/issues">Report Bug</a>
    ·
    <a href="https://github.com/blopker/codebook/issues">Request Feature</a>
  </p>
</div>

## About

Codebook is a spellchecker for code. It binds together the venerable Tree Sitter, the spell checker Spellbook and includes a Language Server for use in any editor. Everything is done in Rust to keep response times snappy.

## Status

Codebook is being developed and not yet ready for public (or private, really) use. Hit the Star button to follow for updates though.

## Features

### Code-aware spell checking

Codebook will only check the parts of your code where a normal linter wouldn't. Comments, string literals and variable definitions for example. Codebook knows how to split camel case and snake case variables, and makes suggestions in the original case.

### Language Server

Codebook comes with a language server. Originally developed for the Zed editor, this language server can be integrated into any editor that supports the language server protocol.

### Dictionary Management

Codebook comes with a dictionary manager, which will automatically download and cache dictionaries for a large number of written languages.

## Running

Currently there are only tests.

Run them with `make test`.

![screenshot](assets/screenshot.png)

## Acknowledgments
- Harper: https://writewithharper.com/
- Harper Zed: https://github.com/Stef16Robbe/harper_zed
- Spellbook: https://github.com/helix-editor/spellbook
- cSpell for VSCode: https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker
- Vale: https://github.com/errata-ai/vale-ls
- TreeSitter Visualizer: https://intmainreturn0.com/ts-visualizer/
