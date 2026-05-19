# Contributing to Codebook

Thanks for your interest in improving Codebook! This document covers the most common contribution paths: adding dictionaries, adding programming language support, running the test suite, and cutting a release.

## Running Tests

Run tests with `make test` after cloning. Integration tests are also available with `make integration_test`, but requires BunJS to run.

## Adding a New Dictionary

Dictionaries in Codebook are currently hardcoded in the dictionary repository file at `crates/codebook/src/dictionaries/repo.rs`.

To add a new Hunspell-compatible dictionary:

1. Open `crates/codebook/src/dictionaries/repo.rs`

1. Locate the `HUNSPELL_DICTIONARIES` static vector (for Hunspell dictionaries) or `TEXT_DICTIONARIES` (for plain text word lists)

1. Add a new entry using the appropriate constructor. For Hunspell dictionaries:

   ```rs /dev/null/example.rs#L1-5
   HunspellRepo::new(
       "nl_nl",  // Dictionary name in snake_case
       "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/nl/index.aff",
       "https://raw.githubusercontent.com/wooorm/dictionaries/refs/heads/main/dictionaries/nl/index.dic",
   ),
   ```

1. Follow the naming convention:
   - Use `snake_case` format (e.g., `en_us`, `nl_nl`, `pt_br`)
   - Language codes should be lowercase
   - Names must be unique across all dictionaries

1. Find dictionary sources:
   - [wooorm/dictionaries](https://github.com/wooorm/dictionaries) - Large collection of Hunspell dictionaries
   - [streetsidesoftware/cspell-dicts](https://github.com/streetsidesoftware/cspell-dicts) - CSpell dictionary collection
   - Both `.aff` (affix rules) and `.dic` (word list) files are required for Hunspell dictionaries

1. (Optional) Run the tests to verify your addition:
   ```bash
   cargo test test_dictionary_names_unique_and_snake_case
   ```
   This test ensures dictionary names are unique and follow the snake_case convention.

For plain text dictionaries, use `TextRepo::new()` instead and add to `TEXT_DICTIONARIES`.

## Adding New Programming Language Support

See the [query development guide](crates/codebook/src/queries/README.md) for instructions on adding Tree-sitter queries for new languages, the tag naming convention, and tips for writing effective queries.

## Release

To publish a new version:

1. Update and commit changelog with new version number
1. Run `make release-lsp`
1. Follow instructions
1. Wait for Actions to finish
1. Go to GitHub Releases
1. Un-mark "prerelease" and publish
1. Run `make publish_crates` to upload to crates.io
