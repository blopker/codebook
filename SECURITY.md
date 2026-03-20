# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Codebook, please report it responsibly by emailing the maintainers or by opening a [GitHub security advisory](https://github.com/blopker/codebook/security/advisories/new).

**Please do not open a public issue for security vulnerabilities.**

## Scope

Codebook is a spell checker that runs locally. It does not send file contents to remote servers. The main areas where security concerns may apply:

- **Dictionary downloads**: Codebook downloads dictionary files from remote URLs on first use and caches them locally. These URLs are hardcoded in the source.
- **Tree-sitter parsers**: Codebook uses tree-sitter grammars to parse source code. These are compiled into the binary.
- **Configuration files**: Codebook reads `codebook.toml` files from the project directory and global config directory.

## Supported Versions

Security fixes are applied to the latest release only.
