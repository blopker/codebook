[Unreleased]

- Actually add Italian (`it`) dictionary, oops.
- Faster CI for Windows

[0.2.8]

- Add Haskell support
- Add Italian (`it`) dictionary
- Add French (`fr`) dictionary
- Don't show suggestions for diagnostics that aren't from Codebook
- Fix duplicate suggestions
- Add "Add to global dictionary action"
- Don't write default settings to config files
- More robust download logic
- Add LTO compile flag to make Codebook even faster
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
- Add support for Ruby (Thanks @vitallium!)

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
