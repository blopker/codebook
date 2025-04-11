# Technical Specification: `codebook-config` Crate

## Overview

The `codebook-config` crate provides a hierarchical configuration management system for the Codebook application, supporting both global (user-level) and project-specific configurations. The system respects platform-specific configuration directories and implements proper precedence rules between the two configuration levels.

## Core Components

### `CodebookConfig` Struct

The main configuration container with the following capabilities:

- **Hierarchical Configuration Management**:
  - Loads global configuration from platform-specific config directories
  - Loads project-specific configuration from the current directory or parent directories
  - Properly merges configurations with project settings taking precedence

- **Configuration Storage**:
  - `project_settings`: The project-specific configuration
  - `global_settings`: The global (user-level) configuration
  - `effective_settings`: The merged result used for actual operations

- **Configuration Discovery and Loading**:
  - Automatically finds global config in platform-specific locations
  - Recursively searches for project config files (`codebook.toml` or `.codebook.toml`)
  - Properly handles missing or invalid configuration files

### `ConfigSettings` Struct

The data model for configuration settings:

- `dictionaries`: List of dictionary IDs for spell-checking
- `words`: Custom allowlist of words that should be considered correct
- `flag_words`: Words that should always be flagged as problematic
- `ignore_paths`: Glob patterns for file paths to exclude
- `ignore_patterns`: Regex patterns for text content to ignore
- `use_global`: Whether to incorporate global configuration (project-config only)

## Key Features

### Global Configuration Support

- **Platform-Specific Paths**:
  - Linux/macOS: `$XDG_CONFIG_HOME/codebook/codebook.toml` if XDG_CONFIG_HOME is set
  - Linux/macOS fallback: `~/.config/codebook/codebook.toml`
  - Windows: `%APPDATA%\codebook\codebook.toml`

- **Configuration Precedence**:
  - Project configuration overrides global configuration
  - Global configuration is loaded first, then extended/overridden by project settings
  - Project config can entirely ignore global config via `use_global = false`

### Configuration Management

- **Read-Only Global Config**: Global configuration is never modified by the library
- **Project-Only Modifications**: Methods like `add_word()` only affect project configuration
- **Global-Only Modifications**: Methods like `add_word_global()` only affect global configuration
- **Effective Settings**: All validation methods use the merged effective settings
- **Transparent Reloading**: Changes to either global or project config are detected on reload

### Case-Insensitive Word Management

All word lists are stored and matched in lowercase, ensuring consistent behavior regardless of capitalization.

### Pattern-Based Ignoring

- **Path Ignoring**: Uses glob patterns for ignoring specified file paths
- **Content Ignoring**: Uses regular expressions for ignoring specified content patterns
- **Lazy RegexSet Initialization**: Compiles regex patterns only when needed

### Error Handling

Uses standard `std::io::Error` with appropriate context for all operations, without external dependencies for error handling.

## Technical Details

### Dependencies

- `serde` with `derive` feature: For serialization/deserialization
- `toml`: For TOML parsing and generation
- `dirs`: For finding platform-specific home directories
- `glob`: For file path pattern matching
- `log`: For logging operations
- `regex`: For text pattern matching

### Thread Safety

Uses `RwLock` for interior mutability, allowing thread-safe access to configuration settings.

### Cache Management

Maintains a temporary cache directory that can be cleaned as needed.

## Integration Points

The crate exposes the following main integration points:

1. `CodebookConfig::load()`: Load both global and project configurations
3. Core validation methods:
   - `is_allowed_word()`
   - `should_flag_word()`
   - `should_ignore_path()`
4. Configuration manipulation methods:
   - `add_word()`: Add words to project allowlist only
   - `add_word_global()`: Add words to global allowlist only
   - `save()`: Save project configuration only
   - `save_global()`: Save global configuration only
   - `reload()`: Reload both global and project configurations

## Usage Example

```rust
// Load configurations (both global and project)
let config = CodebookConfig::load(None)?;

// Check if a path should be ignored (uses effective settings)
let should_ignore = config.should_ignore_path("target/debug/build");

// Check if a word is allowed (uses effective settings)
let is_allowed = config.is_allowed_word("rustc");

// Add a new word to the project allowlist (project config only)
config.add_word("newterm")?;
config.save()?;

// Add a new word to the global allowlist (global config only)
config.add_word_global("newterm")?;
config.save_global()?;

// Clean the cache directory
config.clean_cache();
```

## Configuration File Format

The configuration files use TOML format with the following structure:

```toml
# List of dictionaries to use for spell checking
dictionaries = ["en_us", "en_gb"]

# Custom allowlist of words
words = ["codebook", "rustc"]

# Words that should always be flagged
flag_words = ["todo", "fixme"]

# Glob patterns for paths to ignore
ignore_paths = ["target/**/*", "**/*.md"]

# Regex patterns for text to ignore
ignore_patterns = ["^[A-Z0-9]+$", "\\d{3}-\\d{2}-\\d{4}"]

# Whether to use global configuration (project config only)
use_global = true
```

## Implementation Details

- Word lists are case-insensitive and automatically converted to lowercase for consistent matching
- When merging global and project configurations, duplicate entries are automatically removed
- Project configuration takes precedence over global configuration for all settings
- Regular expressions are compiled lazily to improve performance
- Configuration changes are detected by comparing content, not just timestamps
