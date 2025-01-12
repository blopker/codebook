use tree_sitter::Language;

// Use https://intmainreturn0.com/ts-visualizer/ to help with writing grammar queries
pub static LANGUAGE_SETTINGS: [LanguageSetting; 7] = [
    LanguageSetting {
        name: "rust",
        query: r#"
                (function_item
                    name: (identifier) @identifier)
                (parameter
                    pattern: (identifier) @identifier)
                (let_declaration
                    pattern: (identifier) @identifier)
                (string_content) @string
                (char_literal) @string
                "#,
        extensions: &["rs"],
    },
    LanguageSetting {
        name: "python",
        query: r#"
            (identifier) @identifier
            (comment) @comment
            (string) @string
            (decorated_definition) @identifier
            (function_definition
                name: (identifier) @identifier)
            (class_definition
                name: (identifier) @identifier)
                "#,
        extensions: &["py"],
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
        extensions: &["js"],
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

                "#,
        extensions: &["ts"],
    },
    LanguageSetting {
        name: "html",
        query: r#"
            (text) @string
            (comment) @comment
            (quoted_attribute_value) @string
            "#,
        extensions: &["html", "htm"],
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
        extensions: &["css"],
    },
    LanguageSetting {
        name: "go",
        query: r#"
                (comment) @comment
                (argument_list (interpreted_string_literal) @string)
                (function_declaration (identifier) @identifier)
                (raw_string_literal) @raw_string
                "#,
        extensions: &["go"],
    },
];

#[derive(Debug)]
pub struct LanguageSetting {
    pub query: &'static str,
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

impl LanguageSetting {
    pub fn language(&self) -> Option<Language> {
        match self.name {
            "rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "python" => Some(tree_sitter_python::LANGUAGE.into()),
            "javascript" => Some(tree_sitter_javascript::LANGUAGE.into()),
            "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
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
