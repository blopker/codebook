# Adding New Language Support to Codebook

## LLM Guide for Adding Programming Language Support

This document provides a systematic approach for adding new programming language support to Codebook. Follow these steps in order.

## Prerequisites

- Tree-sitter grammar package name and version for the target language
- Access to the language's tree-sitter repository (usually on GitHub)
- Understanding of the language's syntax basics

## Step-by-Step Process

### 1. Research the Tree-sitter Grammar

Before starting, gather information:

- **Grammar repository**: Find the official tree-sitter grammar repository (e.g., `https://github.com/tree-sitter-grammars/tree-sitter-LANGUAGE`)
- **Package name**: Identify the exact crate name (e.g., `tree-sitter-zig`)
- **Version**: Determine the version to use (check crates.io or user specification)
- **Node types**: Fetch the `queries/highlights.scm` file from the repository to understand node structure

**Key files to examine in the grammar repository:**
- `queries/highlights.scm` - Shows what node types exist
- `src/node-types.json` - Complete node type definitions (if available)
- Example code in the repository's tests

### 2. Add Workspace Dependency

Edit `Cargo.toml` (workspace root):

```toml
[workspace.dependencies]
# ... existing dependencies ...
tree-sitter-LANGUAGE = "VERSION"
```

**Example:**
```toml
tree-sitter-zig = "1.1.2"
```

### 3. Add Crate Dependency

Edit `crates/codebook/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
tree-sitter-LANGUAGE.workspace = true
```

Add in alphabetical order with other tree-sitter dependencies.

### 4. Update Language Type Enum

Edit `crates/codebook/src/queries.rs`:

Add variant to `LanguageType` enum in **alphabetical order**:

```rust
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LanguageType {
    Bash,
    C,
    // ... other languages ...
    YourLanguage,  // Add here
    Zig,
}
```

### 5. Add Language Setting

In `crates/codebook/src/queries.rs`, add entry to `LANGUAGE_SETTINGS` array:

```rust
LanguageSetting {
    type_: LanguageType::YourLanguage,
    ids: &["language_id"],  // LSP language identifier
    dictionary_ids: &["language_id"],  // Dictionary lookup
    query: include_str!("queries/yourlanguage.scm"),
    extensions: &["ext1", "ext2"],  // File extensions
},
```

**Important notes:**
- `ids`: Language identifiers from [VSCode language identifiers](https://code.visualstudio.com/docs/languages/identifiers)
- `extensions`: Common file extensions without the dot
- Place in the array (order doesn't matter functionally but keep consistent)

### 6. Add Language Function Match Arm

In `crates/codebook/src/queries.rs`, update the `language()` method in `impl LanguageSetting`:

```rust
pub fn language(&self) -> Option<Language> {
    match self.type_ {
        // ... existing matches ...
        LanguageType::YourLanguage => Some(tree_sitter_language::LANGUAGE.into()),
        // OR if the crate has a function:
        LanguageType::YourLanguage => Some(tree_sitter_language::language().into()),
    }
}
```

**Note:** Check the tree-sitter crate's API. Most expose either:
- `LANGUAGE` constant (older style)
- `language()` function (newer style)
- `LANGUAGE_TYPENAME` for multi-language crates (e.g., `LANGUAGE_PHP`, `LANGUAGE_TYPESCRIPT`)

### 7. Create Tree-sitter Query File

Create `crates/codebook/src/queries/yourlanguage.scm`

**Query file structure:**
```scheme
; Comments - capture all comment types
(line_comment) @comment
(block_comment) @comment
(doc_comment) @comment

; Identifiers - capture DEFINITIONS only, not usages
(function_declaration
  name: (identifier) @identifier)

(variable_declaration
  (identifier) @identifier)

(parameter
  (identifier) @identifier)

; Struct/Type definitions
(struct_declaration
  name: (type_identifier) @identifier)

(field_declaration
  name: (field_identifier) @identifier)

; String literals - capture string content
(string_content) @string
(string) @string
```

**Critical guidelines:**
- Focus on **definitions**, not references/usages
- Capture user-defined names, not keywords
- Include comments (all types)
- Include string literals
- Test the query thoroughly - invalid queries will fail compilation

**How to discover node types:**
1. Visit the grammar's GitHub repository
2. Check `queries/highlights.scm` for existing patterns
3. Use [Tree-sitter Playground](https://tree-sitter.github.io/tree-sitter/playground.html) to test
4. Copy sample code, paste into playground with your grammar
5. Inspect the AST structure to identify node types

**Common node type patterns by language:**
- Identifiers: `identifier`, `IDENTIFIER`, `name`
- Strings: `string`, `string_content`, `string_literal`
- Comments: `comment`, `line_comment`, `block_comment`, `doc_comment`
- Functions: `function_declaration`, `function_definition`, `FnProto`
- Variables: `variable_declaration`, `var_decl`, `VarDecl`

### 8. Create Example File

Create `examples/example.LANGUAGE` with intentional spelling errors:

**Requirements:**
- Must contain at least one spelling error (for integration tests)
- Include various language constructs: functions, variables, comments, strings
- Use realistic code patterns
- Include misspellings in: identifiers, strings, comments

**Example structure:**
```language
// Comment with speling error
const myVarible = "Hello Wolrd";

function processDatta(inputt) {
    const resullt = inputt + 1;
    return resullt;
}
```

### 9. Create Test File

Create `crates/codebook/tests/test_yourlanguage.rs`:

**Template:**
```rust
use codebook::{
    parser::{TextRange, WordLocation},
    queries::LanguageType,
};

mod utils;

#[test]
fn test_yourlanguage_location() {
    utils::init_logging();
    let sample_text = r#"
// Your sample code with misspellings
const speling = "error";
"#;

    let expected = vec![
        WordLocation::new(
            "speling".to_string(),
            vec![TextRange {
                start_byte: 6,  // Calculate exact byte positions
                end_byte: 13,
            }],
        ),
        // Add more expected misspellings
    ];

    let not_expected = ["const", "std"];  // Keywords that should NOT be flagged

    let processor = utils::get_processor();
    let misspelled = processor
        .spell_check(sample_text, Some(LanguageType::YourLanguage), None)
        .to_vec();

    println!("Misspelled words: {misspelled:?}");

    for e in &expected {
        println!("Expecting: {e:?}");
        let miss = misspelled.iter().find(|r| r.word == e.word).unwrap();
        // locations may be in different orders since they are deduplicated using a HashSet
        assert!(miss.locations.len() == e.locations.len());
        for location in &miss.locations {
            assert!(e.locations.contains(location));
        }
    }

    for result in misspelled {
        assert!(!not_expected.contains(&result.word.as_str()));
    }
}
```

**Test requirements:**
- Include multiple types of misspellings
- Verify byte positions are exact
- Ensure keywords are NOT captured
- Test comments, strings, and identifiers separately

### 10. Run Tests

Execute in order:

```bash
# 1. Verify query is valid
cargo test -p codebook queries::tests::test_all_queries_are_valid

# 2. Run language-specific test
cargo test -p codebook test_yourlanguage

# 3. Run all tests
cargo test -p codebook
```

## Common Issues and Solutions

### Issue: CamelCase words are getting split

- Codebook processing splits words based on common word boundaries in programming like CamelCase and snake_case. Expect that when making tests.

### Issue: Invalid query error with node type

**Error:** `QueryError { message: "NodeTypeName", kind: NodeType }`

**Solution:**
- The node type doesn't exist in the grammar
- Check the grammar's `queries/highlights.scm` for correct node names
- Node types are case-sensitive
- Use tree-sitter playground to verify AST structure

### Issue: Capturing too many or too few occurrences

**Problem:** Test fails because word appears more times than expected

**Solution:**
- Refine query to capture only definitions, not usages
- Use field names in captures: `name: (identifier)` instead of just `(identifier)`
- Check if you're capturing both definition and reference

### Issue: Keywords being captured

**Problem:** Language keywords appear in misspelled words

**Solution:**
- Don't capture `(keyword)` nodes
- Be specific in queries - use parent node context
- Only capture user-defined names

### Issue: Wrong language() function syntax

**Error:** Compilation error in `language()` match arm

**Solution:**
- Check the tree-sitter crate documentation
- Try: `LANGUAGE.into()`, `language().into()`, or `LANGUAGE_VARIANT.into()`
- Look at the crate's lib.rs for the public API

## Testing Checklist

Before considering the implementation complete:

- [ ] Query file compiles without errors
- [ ] `test_all_queries_are_valid` passes
- [ ] Language-specific test passes
- [ ] Example file exists with intentional errors
- [ ] All expected misspellings are caught
- [ ] No keywords are captured
- [ ] Byte positions in tests are accurate
- [ ] Comments are captured
- [ ] String literals are captured
- [ ] Identifier definitions are captured

## File Modification Summary

Files that MUST be modified:

1. `Cargo.toml` - Add workspace dependency
2. `crates/codebook/Cargo.toml` - Add crate dependency
3. `crates/codebook/src/queries.rs` - Add enum variant, setting, and language match
4. `crates/codebook/src/queries/LANGUAGE.scm` - Create query file
5. `examples/example.LANGUAGE` - Create example file
6. `crates/codebook/tests/test_LANGUAGE.rs` - Create test file

## Query File Best Practices

1. **Start simple**: Begin with basic captures (comments, simple identifiers)
2. **Test incrementally**: Add one capture type at a time
3. **Use field names**: `name: (identifier)` is better than `(identifier)`
4. **Check highlights.scm**: The language's highlight query is your best reference
5. **Avoid ambiguity**: Be specific about what context you're capturing
6. **Comment your queries**: Explain what each section captures

## Example: Real Implementation Reference

For a complete reference implementation, examine existing languages:
- Simple: `queries/go.scm`, `tests/test_go.rs`
- Complex: `queries/rust.scm`, `tests/test_rust.rs`
- With strings: `queries/python.scm`, `tests/test_python.rs`

## Byte Position Calculation

Tests require exact byte positions. To calculate:

1. Copy your sample text exactly as in the test
2. Count UTF-8 bytes from start of string to word start
3. Count UTF-8 bytes from start of string to word end
4. Remember: Most ASCII characters are 1 byte, but check UTF-8 encoding

**Pro tip**: Print actual results first, then use those byte positions in your test expectations.

## Final Verification

Run the full test suite:
```bash
cargo test -p codebook
```

All tests should pass. If not, review error messages and adjust queries or test expectations.
