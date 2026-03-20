# Codebook Architecture Refactor

## Goal

Restructure the `codebook` crate internals to support multi-language files (markdown with code blocks, Astro/Vue/Svelte, HTML with `<script>`/`<style>`) and lay groundwork for control comments, custom dictionaries, and a CLI. No public LSP protocol changes needed — the refactor is internal to the `codebook` and `codebook-config` crates.

## Current Architecture

```
LSP Backend
    → Codebook::spell_check(text, ONE language, file_path)
        → resolve_language()           // pick one LanguageType
        → get_dictionaries()           // load dicts for that one language
        → parser::find_locations()     // do everything in one function:
            ├─ Text path: word-boundary split entire text
            └─ Code path: tree-sitter parse + query + word extract + dict check
        → return Vec<WordLocation>
```

### Problems

1. **`find_locations` does too much.** It parses, queries, extracts words, applies skip patterns, and checks dictionaries — all in one function. You can't insert new stages (control comments, injection) without forking the function.

2. **One language per file.** `spell_check` resolves a single `LanguageType` and uses it for the entire file. No way to handle embedded languages.

3. **Dictionary selection is coupled to language resolution.** Dictionaries are gathered once based on the single resolved language. With multiple languages per file, different regions need different dictionaries.

4. **Skip patterns are applied inconsistently.** For `Text` mode, skip patterns are applied during word extraction. For code mode, skip patterns are applied after word extraction against global byte offsets. Both happen inside `find_locations`.

5. **`LanguageType::Text` is a special case everywhere.** The `Text` variant returns `None` from `language()`, has no `.scm` file, no `LanguageSetting` entry returned by `get_language_setting`, and takes a completely different code path in `find_locations`. It's an implicit "not really a language" variant.

## Proposed Architecture

### Pipeline

```
Codebook::spell_check(text, language, file_path)
    │
    ▼
┌─────────────────────────────┐
│  Stage 1: Region Extraction │  Split file into typed regions
│  (one language per region)  │  Most languages: 1 region = whole file
└─────────────┬───────────────┘  Markdown/HTML/Astro: multiple regions
              │
              ▼
┌─────────────────────────────┐
│  Stage 2: Node Extraction   │  Per region: tree-sitter parse + query
│  (AST nodes to check)       │  Returns tagged text spans
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  Stage 3: Word Extraction   │  Per node: split words, apply skip patterns
│  (candidate words)          │  Uses splitter + TextProcessor
└─────────────┬───────────────┘
              │
              ▼
┌─────────────────────────────┐
│  Stage 4: Word Checking     │  Per word: dictionary lookup + config rules
│  (misspelled words)         │  flag_words, allowed_words, min_length
└─────────────┬───────────────┘
              │
              ▼
         Vec<WordLocation>
```

Each stage is a separate function with clear inputs and outputs. No closures passed between stages — data flows as concrete types.

### Data Types

```rust
/// A region of a file associated with a single language.
/// For most files, there's one region covering the whole file.
/// For multi-language files (markdown, astro, vue), there are multiple.
pub struct TextRegion {
    /// Byte range in the original document
    pub start_byte: usize,
    pub end_byte: usize,
    /// Which language governs this region
    pub language: LanguageType,
}

/// A text span extracted from a tree-sitter query match.
/// Coordinates are in original-document byte offsets.
pub struct TextNode {
    /// Byte range in the original document
    pub start_byte: usize,
    pub end_byte: usize,
    /// The text content of this node
    pub text: String,
    /// The capture tag (e.g. "comment", "string", "identifier.function")
    pub tag: String,
}

/// A candidate word extracted from a TextNode, with its position
/// in original-document byte offsets.
pub struct WordCandidate {
    pub word: String,
    pub start_byte: usize,
    pub end_byte: usize,
}
```

`WordLocation` (the final output) stays the same — it groups all locations of a misspelled word together.

### Stage 1: Region Extraction

```rust
// In a new module: src/regions.rs

/// Extract language regions from a document.
/// For single-language files, returns one region covering the whole text.
/// For multi-language files (markdown, astro, vue, html), returns multiple.
pub fn extract_regions(text: &str, language: LanguageType) -> Vec<TextRegion> {
    match language {
        LanguageType::Markdown => extract_markdown_regions(text),
        // Future: LanguageType::HTML => extract_html_regions(text),
        // Future: LanguageType::Astro => extract_astro_regions(text),
        _ => vec![TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language,
        }],
    }
}
```

**Markdown region extraction** parses with `tree_sitter_md`, walks the tree, and produces regions:
- `paragraph`, `atx_heading`, etc. → `LanguageType::Markdown` region
- `fenced_code_block` with `info_string` "python" → `LanguageType::Python` region
- `fenced_code_block` with unknown/missing info string → skip (no region)

This replaces the current `markdown.scm` query approach. Instead of using tree-sitter queries to filter what markdown nodes to check, region extraction identifies the prose vs code boundary, and each region then goes through the normal stage 2 pipeline for its language.

**Language alias resolution** for info strings:

```rust
/// Map markdown info strings to LanguageType.
/// Handles common aliases beyond what LanguageType::from_str covers.
fn resolve_info_string(info: &str) -> Option<LanguageType> {
    // from_str already handles VS Code language IDs like "rust", "python", "javascript"
    // Add common markdown aliases here
    match info.trim().to_lowercase().as_str() {
        "py" => Some(LanguageType::Python),
        "js" => Some(LanguageType::Javascript),
        "ts" => Some(LanguageType::Typescript),
        "sh" | "zsh" | "fish" => Some(LanguageType::Bash),
        "yml" => Some(LanguageType::YAML),
        "c++" | "cc" | "cxx" | "hpp" => Some(LanguageType::Cpp),
        "cs" => Some(LanguageType::CSharp),
        "rb" => Some(LanguageType::Ruby),
        "rs" => Some(LanguageType::Rust),
        "tex" => Some(LanguageType::Latex),
        other => LanguageType::from_str(other).ok(),
    }
}
```

### Stage 2: Node Extraction

```rust
// Refactored from the tree-sitter parts of find_locations_code in src/parser.rs

/// Extract spellcheckable text nodes from a region using tree-sitter.
/// Returns nodes with byte offsets in original document coordinates.
pub fn extract_nodes(
    document_text: &str,
    region: &TextRegion,
    tag_filter: &dyn Fn(&str) -> bool,
) -> Vec<TextNode> {
    let region_text = &document_text[region.start_byte..region.end_byte];

    match region.language {
        LanguageType::Text => {
            // Plain text: the whole region is one node
            vec![TextNode {
                start_byte: region.start_byte,
                end_byte: region.end_byte,
                text: region_text.to_string(),
                tag: "string".to_string(),
            }]
        }
        LanguageType::Markdown => {
            // Markdown prose regions: treat as plain text
            // (region extraction already stripped out code blocks)
            vec![TextNode {
                start_byte: region.start_byte,
                end_byte: region.end_byte,
                text: region_text.to_string(),
                tag: "string".to_string(),
            }]
        }
        _ => {
            // Code: parse with tree-sitter, run query, extract captured nodes
            extract_nodes_with_treesitter(region_text, region.start_byte, region.language, tag_filter)
        }
    }
}

/// Parse text with tree-sitter and extract nodes matching the language's query.
fn extract_nodes_with_treesitter(
    text: &str,
    base_offset: usize,
    language: LanguageType,
    tag_filter: &dyn Fn(&str) -> bool,
) -> Vec<TextNode> {
    let language_setting = get_language_setting(language)?;

    let tree = {
        let mut cache = PARSER_CACHE.lock().unwrap();
        let parser = cache.entry(language).or_insert_with(|| { /* ... */ });
        parser.parse(text, None).unwrap()
    };

    let lang = language_setting.language().unwrap();
    let query = Query::new(&lang, language_setting.query).unwrap();
    let capture_names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut nodes = Vec::new();

    let mut matches = cursor.matches(&query, tree.root_node(), text.as_bytes());
    while let Some(match_) = matches.next() {
        for capture in match_.captures {
            let tag = &capture_names[capture.index as usize];
            if tag == "language" || !tag_filter(tag) {
                continue;
            }
            let node = capture.node;
            nodes.push(TextNode {
                start_byte: node.start_byte() + base_offset,
                end_byte: node.end_byte() + base_offset,
                text: node.utf8_text(text.as_bytes()).unwrap().to_string(),
                tag: tag.to_string(),
            });
        }
    }
    nodes
}
```

Key change: this function **only** extracts nodes. It does not split words or check dictionaries. The `base_offset` parameter handles coordinate translation for injected regions — node byte offsets from tree-sitter are relative to the parsed text, but we need document-global offsets in the output.

### Stage 3: Word Extraction

```rust
// Refactored from TextProcessor in src/parser.rs

/// Extract candidate words from text nodes, applying skip patterns.
/// All byte offsets are in original document coordinates.
pub fn extract_words(
    document_text: &str,
    nodes: &[TextNode],
    skip_patterns: &[Regex],
) -> Vec<WordCandidate> {
    // Compute skip ranges once against the full document
    let skip_ranges = find_skip_ranges(document_text, skip_patterns);

    let mut candidates = Vec::new();
    for node in nodes {
        let words = split_into_words(&node.text);
        for split_word in words {
            let global_start = split_word.start_byte + node.start_byte;
            let global_end = global_start + split_word.word.len();

            if is_within_skip_range(global_start, global_end, &skip_ranges) {
                continue;
            }

            candidates.push(WordCandidate {
                word: split_word.word.to_string(),
                start_byte: global_start,
                end_byte: global_end,
            });
        }
    }
    candidates
}

/// Split a text node's content into individual words using unicode
/// segmentation and camelCase/snake_case splitting.
/// This combines the existing TextProcessor word boundary logic
/// with the splitter module.
fn split_into_words(text: &str) -> Vec<SplitWord> {
    // existing logic from TextProcessor::collect_split_words
    // + splitter::split
}
```

This is a pure function: text in, words out. No dictionary awareness, no language awareness.

### Stage 4: Word Checking

```rust
/// Check candidate words against dictionaries and config rules.
/// Returns WordLocations for misspelled words, grouping all locations
/// of the same word together.
pub fn check_words(
    candidates: &[WordCandidate],
    dictionaries: &[Arc<dyn Dictionary>],
    config: &dyn CodebookConfig,
) -> Vec<WordLocation> {
    // Deduplicate: group candidates by word text
    let mut word_positions: HashMap<&str, Vec<TextRange>> = HashMap::new();
    for candidate in candidates {
        word_positions
            .entry(&candidate.word)
            .or_default()
            .push(TextRange {
                start_byte: candidate.start_byte,
                end_byte: candidate.end_byte,
            });
    }

    // Check each unique word once
    let mut results = Vec::new();
    for (word, positions) in word_positions {
        if config.should_flag_word(word) {
            results.push(WordLocation::new(word.to_string(), positions));
            continue;
        }
        if word.len() < config.get_min_word_length() {
            continue;
        }
        if config.is_allowed_word(word) {
            continue;
        }
        let is_correct = dictionaries.iter().any(|dict| dict.check(word));
        if !is_correct {
            results.push(WordLocation::new(word.to_string(), positions));
        }
    }
    results
}
```

This replaces the `check_function` closure that's currently threaded through `find_locations`. The closure pattern made it impossible to test word checking independently.

### Orchestration in `Codebook::spell_check`

```rust
pub fn spell_check(
    &self,
    text: &str,
    language: Option<LanguageType>,
    file_path: Option<&str>,
) -> Vec<WordLocation> {
    // ... existing path ignore/include logic ...

    let language = self.resolve_language(language, file_path);

    // Build skip patterns once
    let mut skip_patterns = get_default_skip_patterns().clone();
    if let Some(user_patterns) = self.config.get_ignore_patterns() {
        skip_patterns.extend(user_patterns);
    }

    // Stage 1: Split into language regions
    let regions = regions::extract_regions(text, language);

    // Collect dictionaries for all languages present in the file
    let languages_in_file: Vec<LanguageType> = regions.iter().map(|r| r.language).collect();
    let dictionaries = self.get_dictionaries_for_languages(&languages_in_file);

    // Stages 2-4: Process each region
    let mut all_candidates = Vec::new();
    for region in &regions {
        let nodes = parser::extract_nodes(text, region, &|tag| {
            self.config.should_check_tag(tag)
        });
        let candidates = parser::extract_words(text, &nodes, &skip_patterns);
        all_candidates.extend(candidates);
    }

    // Stage 4: Check all words at once (deduplicates across regions)
    parser::check_words(&all_candidates, &dictionaries, self.config.as_ref())
}
```

### Dictionary Selection Changes

```rust
/// Gather dictionaries for all languages present in a file.
fn get_dictionaries_for_languages(
    &self,
    languages: &[LanguageType],
) -> Vec<Arc<dyn Dictionary>> {
    let mut dictionary_ids: Vec<String> = self.config.get_dictionary_ids();

    // Add language-specific dictionaries for all languages in the file
    for lang in languages {
        dictionary_ids.extend(lang.dictionary_ids());
    }

    // Add defaults
    dictionary_ids.extend(DEFAULT_DICTIONARIES.iter().map(|f| f.to_string()));

    // Deduplicate
    dictionary_ids.sort();
    dictionary_ids.dedup();

    dictionary_ids
        .iter()
        .filter_map(|id| self.manager.get_dictionary(id))
        .collect()
}
```

This replaces the current `get_dictionaries(Option<LanguageType>)` which only handles one language.

## Module Layout After Refactor

```
codebook/src/
├── lib.rs              # Codebook struct, spell_check orchestration
├── regions.rs          # NEW: Stage 1 — region extraction
├── parser.rs           # Stages 2+3 — node extraction, word extraction
├── checker.rs          # NEW: Stage 4 — word checking
├── splitter.rs         # Word splitting (camelCase, snake_case) — unchanged
├── regexes.rs          # Skip patterns — unchanged
├── queries.rs          # LanguageType, LanguageSetting, .scm files — unchanged
├── queries/            # .scm query files — unchanged
└── dictionaries/       # Dictionary loading — unchanged
```

Key moves:
- `find_locations` and `find_locations_code` in `parser.rs` → split into `extract_nodes` + `extract_words`
- Dictionary checking logic currently in `Codebook::spell_check` closure → `checker.rs::check_words`
- Region extraction → new `regions.rs` module
- `TextProcessor` stays in `parser.rs` but is simplified — it only does word extraction now, no dictionary checking

## What Gets Deleted

- `parser::find_locations()` — replaced by the pipeline orchestration in `Codebook::spell_check`
- `parser::find_locations_code()` — split into `extract_nodes` + `extract_words`
- `TextProcessor::process_words_with_check()` — word checking moves to stage 4
- `dictionary::find_locations_with_dictionary_batch()` — unused after refactor
- `queries/markdown.scm` — markdown region extraction replaces the query approach
- The `check_function` closure pattern — replaced by concrete `check_words` function

## What Stays the Same

- All `.scm` query files (except `markdown.scm`)
- `LanguageType` enum and `LANGUAGE_SETTINGS` table
- `LanguageSetting` struct and `language()` method
- `splitter::split()` — word splitting logic
- `regexes.rs` — skip patterns
- `dictionaries/` — all dictionary types, manager, repo
- `CodebookConfig` trait and `CodebookConfigFile` implementation
- `CodebookConfigMemory` for tests
- The LSP crate (`codebook-lsp`) — `Backend`, `LanguageServer` impl, all commands
- `Codebook::spell_check` signature (takes same args, returns same type)
- `Codebook::get_suggestions` — unchanged
- `WordLocation`, `TextRange` — unchanged

## Implementation Order

This can be done incrementally, keeping tests green at each step:

### Step 1: Introduce data types and stage functions as wrappers

Add `TextRegion`, `TextNode`, `WordCandidate` types. Write `extract_regions`, `extract_nodes`, `extract_words`, `check_words` as new functions that internally call the existing `find_locations` code. Write tests for each stage function independently. Don't delete anything yet.

### Step 2: Rewire `Codebook::spell_check` to use the pipeline

Replace the body of `spell_check` with the pipeline orchestration. It should call the stage functions instead of `find_locations` directly. All existing integration tests should still pass since the external behavior is the same.

### Step 3: Inline and delete old code

Now that nothing calls `find_locations` or `find_locations_code`, move their internal logic into the stage functions and delete the old functions. Remove `TextProcessor::process_words_with_check` (keep `extract_words` and `collect_split_words`). Remove `find_locations_with_dictionary_batch`.

### Step 4: Implement markdown region extraction

Replace the current markdown.scm query approach with proper region extraction:
- Parse markdown with `tree_sitter_md`
- Walk the AST to identify prose regions and fenced code blocks
- Map info strings to `LanguageType` using `resolve_info_string`
- Delete `markdown.scm`

This makes markdown code blocks spell-checked with the correct language grammar and dictionaries.

### Step 5: Update tests

- Existing integration tests (test_markdown.rs, test_python.rs, etc.) should pass unchanged
- Add new unit tests for each stage function
- Add integration tests for markdown with code blocks in different languages
- Add test for unknown info strings (should be skipped, not crash)

## Future Work (Not Part of This Refactor)

These features are enabled by the pipeline architecture but should be done in separate passes:

### Control Comments

Add a filtering step between stage 2 (node extraction) and stage 3 (word extraction). Scan for comments matching patterns like:
- `// codebook:ignore-next-line` — add the next line's byte range to skip ranges
- `// codebook:ignore-start` / `// codebook:ignore-end` — add enclosed range to skip ranges
- `// codebook:words word1,word2` — add words to allowed list for this file
- `<!-- codebook:ignore -->` — HTML/markdown variant

This works naturally because nodes already carry byte offsets and tags. A comment node with tag "comment" containing "codebook:ignore-next-line" can compute the next line's byte range and add it to the skip ranges that stage 3 uses.

For file-level directives (`codebook:ignore-file`), short-circuit before stage 1.

### Custom User Dictionaries

Changes needed in `codebook-config` and `dictionaries/`:

1. Add `custom_dictionaries` field to `ConfigSettings`:
   ```toml
   # codebook.toml
   [custom_dictionaries]
   my_project = "path/to/project-words.txt"
   medical = "path/to/medical-terms.dic"
   ```

2. `DictionaryManager::get_dictionary` should check for local file paths in addition to the `get_repo` lookup. If `id` maps to a path in config, load it as a `TextDictionary` (for `.txt`) or `HunspellDictionary` (for `.dic`/`.aff` pairs).

3. Relative paths should resolve from the project config file's directory.

No pipeline changes needed — custom dictionaries just appear in the dictionary list alongside built-in ones.

### Astro/Vue/Svelte Support

Same pattern as markdown region extraction:

1. Add `tree-sitter-astro`, `tree-sitter-vue`, etc. as dependencies
2. Add `LanguageType::Astro`, `LanguageType::Vue`, `LanguageType::Svelte`
3. Write `extract_astro_regions`, `extract_vue_regions`, etc. in `regions.rs`
4. These parse the file, identify `<script>`, `<template>`, `<style>` sections, and produce regions with appropriate language types

The `.scm` query files for the embedded languages (TypeScript, HTML, CSS) already exist and work unchanged — they're used in stage 2 for each region.

### HTML `<script>` and `<style>` Injection

Same pattern. `extract_html_regions` would identify `<script>` and `<style>` tags and create JavaScript/CSS regions. The rest of the HTML becomes HTML regions checked with `html.scm`.

### Korean/Asian Language Support

Affects stage 3 (word extraction) only. The current `split_into_words` uses `unicode-segmentation`'s `split_word_bound_indices`, which works for space-separated languages. For Korean/CJK, options:

1. **Syllable-level checking** — Unicode word boundaries do produce Hangul syllable blocks, so basic Korean may work with the existing splitter. Test this first.
2. **Segmentation library** — if syllable-level is too granular, integrate a word segmentation library. Stage 3 would check the script of the text and choose the appropriate splitter.
3. **Dictionary support** — need Hunspell dictionaries for these languages. The dictionary system already supports this (just add entries to `HUNSPELL_DICTIONARIES`).

No pipeline architecture changes needed — just a different word splitting strategy in stage 3.

### CLI for CI

New crate: `codebook-cli`. Uses the `codebook` crate directly:

```rust
// codebook-cli/src/main.rs
fn main() {
    let args = parse_args();
    let config = CodebookConfigFile::load(Some(&args.project_dir))?;
    let codebook = Codebook::new(Arc::new(config))?;

    let mut exit_code = 0;
    for file in discover_files(&args) {
        let results = codebook.spell_check_file(&file);
        if !results.is_empty() {
            exit_code = 1;
            format_results(&file, &results, args.format);
        }
    }
    std::process::exit(exit_code);
}
```

Output formats: `text` (human readable), `json` (machine readable), `sarif` (GitHub Actions).

File discovery: walk directory, respect `.gitignore` + config `ignore_paths`/`include_paths`.

No pipeline changes needed — the CLI uses `Codebook::spell_check_file` which already returns `Vec<WordLocation>`.
