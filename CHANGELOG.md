[0.3.7]

- Better error message for user-supplied regex
- Improve Go support
- Add query for Ruby method names

[0.3.6]

- Use rustls_platform_verifier to let Codebook use OS cert chain for users with proxies
- Add `pt_br` dictionary.
- Add support for Rust's comment blocks

[0.3.5]

- Add support for Java

[0.3.4]

- Do not announce `diagnostic_provider` capability (PR #86)

[0.3.3]

- Fix the cargo release by moving test example files

[0.3.2]

- Pin softprops/action-gh-release action since 2.3.0 broke CI

[0.3.1]

- Updated the global word list to include more common programming terms.
- Many small performance improvements to reduce allocations. On a M1 Mac checking the full Beowulf text went from ~80ms to ~13ms 🚀

[0.3.0]

Breaking changes:

- User defined regex is now run on a file line-by-line instead of word-by-word. This means regex should likely not match the beginning of a line. For example to match DNA, this pattern used to work: `^[ATCG]+$`. This pattern will now need to be something like: `\\b[ATCG]+\\b` (double `\\` is for escaping in TOML)

- Codebook will now ignore text like URLs and color hex codes by default. See README `User-Defined Regex Patterns` for more details.

[0.2.13]

- Switch out OpenSSL for rustls
- Improve Go query to support short hand variables

[0.2.12]

- Fix #72: Update Spellbook

[0.2.11]

- Add Swedish dictionary (`sv`)
- Fix #69 (nice) directory structure not being created for global config

[0.2.10]

- Fix #67: Update Spellbook

[0.2.9]

- Actually add Italian (`it`) dictionary, oops.
- Faster CI for Windows
- Don't strip binaries

[0.2.8]

- Add Haskell support
- Add Italian (`it`) dictionary
- Add French (`fr`) dictionary
- Don't show suggestions for diagnostics that aren't from Codebook
- Fix duplicate suggestions
- Add "Add to global dictionary action"
- Don't write default settings to config files
- More robust download logic
- Add LTO compile flag to make Codebook even faster 🚀
- Remove GLIBC builds (use musl!)
- Add logging and make it configurable

[0.2.7]

- Add German dictionaries (`de`, `de_at`, and `de_ch`)
- Add support for R

[0.2.6]

- Better error handling for suggestions

[0.2.5]

- Add Russian dictionary (ru)
- Fairly get suggestions from all active dictionaries.
- Add PHP support.
- Fix codebook.toml not being created in new projects on "Add to dictionary".
- JavaScript: Make properties on object definitions check, and try/catch error definitions
- TypeScript: Make properties on object definitions check, try/catch error definitions, and interface support

[0.2.4]

- Make ignore_paths actually work

[0.2.3]

- Handle unicode in a much better way
- Add support for Ruby

[0.2.2]

- Fix a char boundary issue
- Add ES and EN_GB dictionaries that actually work

[0.2.0]

- Rework config to allow for global config.
- Ignore words less than 3 chars.
- Remake metadata file if it is corrupt.
- Protect against deleted cached files.

[0.1.22]

- Better support for TypeScript classes and fields

[0.1.21]

- Better Python support

[0.1.20]

- Fix CI

[0.1.19]

- Add support for C

[0.1.18]

- Add `ignore_patterns` for a list of regex expressions to use when checking words. Any matches will ignore the word being checked.

[0.1.17]

- Added a download manager for adding many different dictionaries later
- Using a larger en_us dictionary as default
- Now checks on every change, instead of on save. May add an option later to toggle this off
- Add a command to the LSP binary to clear cache
- Don't give a code action when a word is not misspelled
- Vendor OpenSSL
- Add 'software_terms'
- Only lowercase ascii letters when checking

[0.1.15]

- Check words for different cases (#2)
- Improve Golang query
- Add link to change log in release notes

[0.1.14]

- Recheck all open files when config changes

[0.1.13]

- Start of change log!
- Switch to musl for Linux builds (#1)
