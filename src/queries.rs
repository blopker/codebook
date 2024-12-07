use std::collections::HashMap;

use lazy_static::lazy_static;
use tree_sitter::Language;

lazy_static! {
    pub static ref LANGUAGE_QUERIES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert(
            "rust",
            r#"
            (identifier) @identifier
            (string_literal) @string
            (line_comment) @comment
            (block_comment) @comment
            (raw_string_literal) @string
            (char_literal) @string
        "#,
        );
        map.insert(
            "python",
            r#"
            (identifier) @identifier
            (string) @string
            (comment) @comment
            (string_content) @string
            (concatenated_string) @string
            (decorated_definition) @identifier
            (function_definition
                name: (identifier) @identifier)
            (class_definition
                name: (identifier) @identifier)
        "#,
        );
        map.insert(
            "javascript",
            r#"
            (identifier) @identifier
            (string) @string
            (comment) @comment
            (template_string) @string
            (jsx_text) @string
            (property_identifier) @identifier
            (shorthand_property_identifier) @identifier
            (method_definition
                name: (property_identifier) @identifier)
            (class_declaration
                name: (identifier) @identifier)
        "#,
        );
        map.insert(
            "typescript",
            r#"
            (identifier) @identifier
            (string) @string
            (comment) @comment
            (template_string) @string
            (jsx_text) @string
            (property_identifier) @identifier
            (shorthand_property_identifier) @identifier
            (method_definition
                name: (property_identifier) @identifier)
            (class_declaration
                name: (identifier) @identifier)
            (type_identifier) @identifier
            (interface_declaration
                name: (type_identifier) @identifier)
        "#,
        );
        map.insert(
            "html",
            r#"
            (text) @string
            (comment) @comment
            (quoted_attribute_value) @string
        "#,
        );
        map.insert(
            "css",
            r#"
            (class_name) @identifier
            (id_name) @identifier
            (property_name) @identifier
            (comment) @comment
            (string_value) @string
            (plain_value) @identifier
        "#,
        );
        map.insert(
            "go",
            r#"
            (identifier) @identifier
            (interpreted_string_literal) @string
            (raw_string_literal) @string
            (comment) @comment
            (field_identifier) @identifier
            (type_identifier) @identifier
            (package_identifier) @identifier
            "#,
        );

        map
    };
}

#[derive(Debug)]
pub struct LanguageQuery {
    pub query: &'static str,
    pub language: Language,
    // pub language_name: String,
}

fn language_from_name(name: &str) -> Option<Language> {
    match name {
        "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
        "python" => Some(tree_sitter_python::LANGUAGE.into()),
        "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "html" => Some(tree_sitter_html::LANGUAGE.into()),
        _ => None,
    }
}

pub fn get_query(language_name: &str) -> Option<LanguageQuery> {
    match LANGUAGE_QUERIES.get(language_name) {
        Some(query) => Some(LanguageQuery {
            query,
            language: language_from_name(language_name)?,
            // language_name: language_name.to_owned(),
        }),
        None => None,
    }
}
