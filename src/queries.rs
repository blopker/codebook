use lazy_static::lazy_static;
use tree_sitter::Language;

lazy_static! {
    pub static ref LANGUAGE_SETTINGS: Vec<LanguageSetting> = {
        vec![
            LanguageSetting {
                name: "rust",
                query: r#"
                (identifier) @identifier
                (string_literal) @string
                (line_comment) @comment
                (block_comment) @comment
                (raw_string_literal) @string
                (char_literal) @string
                "#,
                extensions: vec!["rs"],
                ignore_node_kinds: vec![],
            },
            LanguageSetting {
                name: "python",
                query: r#"
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
                extensions: vec!["py"],
                ignore_node_kinds: vec![],
            },
            LanguageSetting {
                name: "javascript",
                query: r#"
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
                extensions: vec!["js"],
                ignore_node_kinds: vec![],
            },
            LanguageSetting {
                name: "typescript",
                query: r#"
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
                extensions: vec!["ts"],
                ignore_node_kinds: vec![],
            },
            LanguageSetting {
                name: "html",
                query: r#"
            (text) @string
            (comment) @comment
            (quoted_attribute_value) @string
            "#,
                extensions: vec!["html"],
                ignore_node_kinds: vec![],
            },
            LanguageSetting {
                name: "css",
                query: r#"
            (class_name) @identifier
            (id_name) @identifier
            (property_name) @identifier
            (comment) @comment
            (string_value) @string
            (plain_value) @identifier
            "#,
                extensions: vec!["css"],
                ignore_node_kinds: vec![],
            },
            LanguageSetting {
                name: "go",
                query: r#"
                (comment) @comment
                (argument_list (interpreted_string_literal) @string)
                (function_declaration (identifier) @identifier)
                (raw_string_literal) @raw_string
                "#,
                extensions: vec!["go"],
                ignore_node_kinds: vec![],
            },
        ]
    };
}

#[derive(Debug)]
pub struct LanguageSetting {
    pub query: &'static str,
    pub name: &'static str,
    pub extensions: Vec<&'static str>, // pub language_name: String,
    pub ignore_node_kinds: Vec<&'static str>,
}

impl LanguageSetting {
    pub fn language(&self) -> Option<Language> {
        match self.name {
            "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "python" => Some(tree_sitter_python::LANGUAGE.into()),
            "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
            "html" => Some(tree_sitter_html::LANGUAGE.into()),
            "go" => Some(tree_sitter_go::LANGUAGE.into()),
            _ => None,
        }
    }
}

pub fn get_language_setting(language_name: &str) -> Option<&LanguageSetting> {
    for setting in LANGUAGE_SETTINGS.iter() {
        if setting.name == language_name {
            if setting.language().is_some() {
                return Some(setting);
            }
        }
    }
    None
}

pub fn get_language_name_from_filename(filename: &str) -> String {
    let extension = filename.split('.').last().unwrap();
    for setting in LANGUAGE_SETTINGS.iter() {
        for ext in setting.extensions.iter() {
            if ext == &extension {
                return setting.name.to_string();
            }
        }
    }
    "text".to_string()
}
