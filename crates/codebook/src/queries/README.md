# Tree-sitter Queries

This directory contains Tree-sitter query files (`.scm`) that define which parts of each language's source code are extracted for spell checking.

## Tag Convention

Every capture name is a **tag** that categorizes the matched text. Tags use a dot-separated hierarchy so users can filter what gets checked via `include_tags`/`exclude_tags` in `codebook.toml`. Matching is prefix-based: `"comment"` matches `comment`, `comment.line`, `comment.block`, etc.

### Available Tags

| Capture name | When to use |
| --- | --- |
| `@comment` | Generic comments (when line/block aren't distinguished) |
| `@comment.line` | Line comments (`//`, `#`, `--`, etc.) |
| `@comment.block` | Block comments (`/* */`, `{- -}`, etc.) |
| `@string` | String literals and string content |
| `@string.special` | Atoms, symbols, struct tags, and other non-standard strings |
| `@string.heredoc` | Heredoc bodies |
| `@identifier` | Fallback for ambiguous identifiers that don't fit below |
| `@identifier.function` | Function and method name definitions |
| `@identifier.type` | Type, class, struct, interface, enum name definitions |
| `@identifier.parameter` | Function parameter names |
| `@identifier.field` | Struct field and object property names |
| `@identifier.variable` | Variable declaration names |
| `@identifier.constant` | Constant and enum member names |
| `@identifier.module` | Package, module, and namespace names |

Not every language needs every tag. HTML, for example, only uses `@comment` and `@string`. You can get a feel for which tags are available for a specific language by looking at the `scm` file for that language in this directory.

## Adding a New Language

### 1. Create the Query File

Create a new `.scm` file in this directory named after your language (e.g., `java.scm`).

Use namespaced capture names from the table above. Example:

```scheme
(comment) @comment
(string_content) @string
(function_declaration name: (identifier) @identifier.function)
(parameter name: (identifier) @identifier.parameter)
(variable_declaration (identifier) @identifier.variable)
(class_declaration name: (identifier) @identifier.type)
```

### 2. Understand the Language's AST

Use these tools to explore the grammar's node types:

- [Tree-sitter Playground](https://tree-sitter.github.io/tree-sitter/7-playground.html)
- [Tree-sitter Visualizer](https://blopker.github.io/ts-visualizer/)

A good approach:

1. Write sample code with identifiers, strings, and comments
2. Paste it into the playground/visualizer
3. Observe the node types used for each element
4. Create capture patterns that target only definition nodes, not usages

### 3. Update the Language Settings

Add your language to `queries.rs`:

1. Add a new variant to the `LanguageType` enum
2. Add a new entry to the `LANGUAGE_SETTINGS` array with the language type, file extensions, language identifiers, and path to your query file

### 4. Add the Tree-sitter Grammar

Add the grammar as a dependency in `Cargo.toml` and update the `language()` function in `queries.rs` to return the correct parser.

### 5. Test

```bash
cargo test -p codebook queries::tests::test_all_queries_are_valid
```

Additional language tests go in `crates/codebook/tests/`. Example files with at least one spelling error go in `examples/`.

## Tips

- Focus on capturing **definitions**, not usages
- Only capture nodes that contain user-defined text (not keywords)
- Always use namespaced capture names (`@identifier.function`, not `@func_declaration`)
- Use the most specific tag that fits (e.g., `@identifier.type` over `@identifier`)
- Start simple and add complexity as needed
- Look at existing query files for patterns
