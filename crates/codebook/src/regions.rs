use crate::queries::LanguageType;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{LazyLock, Mutex};
use tree_sitter::Parser;

/// A region of a file associated with a single language.
/// For most files, there's one region covering the whole file.
/// For multi-language files (markdown, astro, vue), there are multiple.
#[derive(Debug, Clone, PartialEq)]
pub struct TextRegion {
    /// Byte range start in the original document
    pub start_byte: usize,
    /// Byte range end in the original document
    pub end_byte: usize,
    /// Which language governs this region
    pub language: LanguageType,
}

/// Parser cache for region extraction (separate from the main parser cache
/// since region extraction uses different grammars/queries than node extraction).
static REGION_PARSER_CACHE: LazyLock<Mutex<HashMap<LanguageType, Parser>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Extract language regions from a document.
/// For single-language files, returns one region covering the whole text.
/// For multi-language files (markdown), returns multiple regions.
pub fn extract_regions(text: &str, language: LanguageType) -> Vec<TextRegion> {
    match language {
        LanguageType::Markdown => extract_markdown_regions(text),
        _ => vec![TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language,
        }],
    }
}

/// Map markdown info strings to LanguageType.
/// Uses LanguageType::from_str which checks ids and file extensions
/// in LANGUAGE_SETTINGS. Returns None for unknown or empty strings.
fn resolve_info_string(info: &str) -> Option<LanguageType> {
    let trimmed = info.trim().to_lowercase();
    if trimmed.is_empty() {
        return None;
    }
    match LanguageType::from_str(&trimmed) {
        Ok(LanguageType::Text) => None, // from_str returns Text for unknown
        Ok(lang) => Some(lang),
        Err(_) => None,
    }
}

/// Extract regions from a markdown file.
/// Prose sections become Markdown regions (treated as plain text in node extraction).
/// Fenced code blocks become regions of the appropriate language.
fn extract_markdown_regions(text: &str) -> Vec<TextRegion> {
    let lang: tree_sitter::Language = tree_sitter_md::LANGUAGE.into();

    let tree = {
        let mut cache = REGION_PARSER_CACHE.lock().unwrap();
        let parser = cache.entry(LanguageType::Markdown).or_insert_with(|| {
            let mut parser = Parser::new();
            parser.set_language(&lang).unwrap();
            parser
        });
        parser.parse(text, None).unwrap()
    };

    let mut regions = Vec::new();
    let root = tree.root_node();
    let provider = text.as_bytes();

    walk_markdown_node(root, provider, &mut regions);

    // Sort by start position
    regions.sort_by_key(|r| r.start_byte);

    // If no regions found (empty file, etc.), return the whole thing as markdown
    if regions.is_empty() {
        return vec![TextRegion {
            start_byte: 0,
            end_byte: text.len(),
            language: LanguageType::Markdown,
        }];
    }

    regions
}

/// Recursively walk markdown AST to find prose and code block regions.
fn walk_markdown_node(node: tree_sitter::Node, source: &[u8], regions: &mut Vec<TextRegion>) {
    match node.kind() {
        "fenced_code_block" => {
            // Find info_string and code_fence_content children
            let mut info_string = None;
            let mut code_content = None;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "info_string" => {
                        // Get the language child of info_string
                        let mut ic = child.walk();
                        for info_child in child.children(&mut ic) {
                            if info_child.kind() == "language" {
                                info_string =
                                    Some(info_child.utf8_text(source).unwrap_or("").to_string());
                            }
                        }
                    }
                    "code_fence_content" => {
                        code_content = Some((child.start_byte(), child.end_byte()));
                    }
                    _ => {}
                }
            }

            if let Some((start, end)) = code_content
                && start < end
                && let Some(info) = info_string
                && let Some(lang) = resolve_info_string(&info)
            {
                regions.push(TextRegion {
                    start_byte: start,
                    end_byte: end,
                    language: lang,
                });
            }
        }
        "html_block" => {
            // Block-level HTML — treat as an HTML region
            if node.start_byte() < node.end_byte() {
                regions.push(TextRegion {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    language: LanguageType::HTML,
                });
            }
        }
        "inline" => {
            // Check parent — we want inline content from paragraphs and headings
            if let Some(parent) = node.parent() {
                match parent.kind() {
                    "paragraph" | "atx_heading" | "setext_heading" => {
                        if node.start_byte() < node.end_byte() {
                            regions.push(TextRegion {
                                start_byte: node.start_byte(),
                                end_byte: node.end_byte(),
                                language: LanguageType::Markdown,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {
            // Recurse into children
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk_markdown_node(child, source, regions);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_language_region() {
        let regions = extract_regions("fn main() {}", LanguageType::Rust);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].language, LanguageType::Rust);
        assert_eq!(regions[0].start_byte, 0);
        assert_eq!(regions[0].end_byte, 12);
    }

    #[test]
    fn test_text_region() {
        let regions = extract_regions("hello world", LanguageType::Text);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].language, LanguageType::Text);
    }

    #[test]
    fn test_markdown_prose_only() {
        let text = "# Hello World\n\nSome paragraph text.\n";
        let regions = extract_regions(text, LanguageType::Markdown);
        assert!(regions.len() >= 2); // heading + paragraph
        for r in &regions {
            assert_eq!(r.language, LanguageType::Markdown);
        }
    }

    #[test]
    fn test_markdown_with_code_block() {
        let text = "# Hello\n\nSome text.\n\n```python\ndef foo():\n    pass\n```\n\nMore text.\n";
        let regions = extract_regions(text, LanguageType::Markdown);
        println!("Regions: {regions:#?}");

        // Should have markdown prose regions + python code region
        let python_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.language == LanguageType::Python)
            .collect();
        assert_eq!(python_regions.len(), 1, "Expected one Python region");

        let md_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.language == LanguageType::Markdown)
            .collect();
        assert!(
            md_regions.len() >= 2,
            "Expected at least 2 markdown prose regions"
        );
    }

    #[test]
    fn test_markdown_unknown_language_skipped() {
        let text = "# Hello\n\n```unknownlang\nsome code\n```\n\nMore text.\n";
        let regions = extract_regions(text, LanguageType::Markdown);
        // Unknown language code block should produce no region
        for r in &regions {
            assert_eq!(r.language, LanguageType::Markdown);
        }
    }

    #[test]
    fn test_markdown_no_info_string_skipped() {
        let text = "# Hello\n\n```\nsome code\n```\n\nMore text.\n";
        let regions = extract_regions(text, LanguageType::Markdown);
        // Code block without info string should produce no region
        for r in &regions {
            assert_eq!(r.language, LanguageType::Markdown);
        }
    }

    #[test]
    fn test_markdown_html_block() {
        let text = "# Hello\n\n<div class=\"foo\">\n  <p>A paragraph</p>\n</div>\n\nMore text.\n";
        let regions = extract_regions(text, LanguageType::Markdown);
        println!("Regions: {regions:#?}");

        let html_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.language == LanguageType::HTML)
            .collect();
        assert_eq!(html_regions.len(), 1, "Expected one HTML region");

        let md_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.language == LanguageType::Markdown)
            .collect();
        assert!(md_regions.len() >= 2, "Expected heading + paragraph prose regions");
    }

    #[test]
    fn test_resolve_info_string_aliases() {
        assert_eq!(resolve_info_string("py"), Some(LanguageType::Python));
        assert_eq!(resolve_info_string("js"), Some(LanguageType::Javascript));
        assert_eq!(resolve_info_string("ts"), Some(LanguageType::Typescript));
        assert_eq!(resolve_info_string("sh"), Some(LanguageType::Bash));
        assert_eq!(resolve_info_string("rs"), Some(LanguageType::Rust));
        assert_eq!(resolve_info_string("rb"), Some(LanguageType::Ruby));
        assert_eq!(resolve_info_string("yml"), Some(LanguageType::YAML));
        assert_eq!(resolve_info_string("c++"), Some(LanguageType::Cpp));
        assert_eq!(resolve_info_string(""), None);
        assert_eq!(resolve_info_string("unknownlang"), None);
    }

    #[test]
    fn test_resolve_info_string_vscode_ids() {
        assert_eq!(resolve_info_string("python"), Some(LanguageType::Python));
        assert_eq!(
            resolve_info_string("javascript"),
            Some(LanguageType::Javascript)
        );
        assert_eq!(resolve_info_string("rust"), Some(LanguageType::Rust));
        assert_eq!(resolve_info_string("bash"), Some(LanguageType::Bash));
        assert_eq!(resolve_info_string("go"), Some(LanguageType::Go));
    }

    #[test]
    fn test_markdown_multiple_code_blocks() {
        let text = "Text.\n\n```bash\nmkdir dir\n```\n\n```python\nx = 1\n```\n\nEnd.\n";
        let regions = extract_regions(text, LanguageType::Markdown);

        let bash_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.language == LanguageType::Bash)
            .collect();
        let python_regions: Vec<_> = regions
            .iter()
            .filter(|r| r.language == LanguageType::Python)
            .collect();

        assert_eq!(bash_regions.len(), 1);
        assert_eq!(python_regions.len(), 1);
    }

    #[test]
    fn test_markdown_code_block_content_correct() {
        let text = "Hello.\n\n```python\ndef foo():\n    pass\n```\n";
        let regions = extract_regions(text, LanguageType::Markdown);
        let py = regions
            .iter()
            .find(|r| r.language == LanguageType::Python)
            .unwrap();
        let content = &text[py.start_byte..py.end_byte];
        assert!(
            content.contains("def foo()"),
            "Expected python code, got: {content:?}"
        );
    }
}
