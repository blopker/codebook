use tree_sitter::Language;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LanguageType {
    Rust,
    Python,
    Javascript,
    Typescript,
    Html,
    Css,
    Go,
    Text,
}

impl LanguageType {
    pub fn from_str(s: &str) -> Option<LanguageType> {
        match s {
            "rust" => Some(LanguageType::Rust),
            "python" => Some(LanguageType::Python),
            "javascript" => Some(LanguageType::Javascript),
            "typescript" => Some(LanguageType::Typescript),
            "html" => Some(LanguageType::Html),
            "css" => Some(LanguageType::Css),
            "go" => Some(LanguageType::Go),
            "text" => Some(LanguageType::Text),
            _ => None,
        }
    }
}

static COMMON_DICTIONARY: &str = include_str!("../../word_lists/combined.gen.txt");
// Use https://intmainreturn0.com/ts-visualizer/ to help with writing grammar queries
pub static LANGUAGE_SETTINGS: [LanguageSetting; 7] = [
    LanguageSetting {
        type_: LanguageType::Rust,
        name: "rust",
        query: r#"
                (function_item
                    name: (identifier) @identifier)
                (parameter
                    pattern: (identifier) @identifier)
                (let_declaration
                    pattern: (identifier) @identifier)
                (struct_item
                    name: (type_identifier) @identifier)
                (field_declaration
                    name: (field_identifier) @identifier)
                (line_comment) @comment
                (string_content) @string
                (char_literal) @string
                "#,
        extensions: &["rs"],
    },
    LanguageSetting {
        type_: LanguageType::Python,
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
        type_: LanguageType::Javascript,
        name: "javascript",
        query: r#"
            (comment) @comment
            (string_fragment) @string
            (string) @string
            (variable_declarator
                name: (identifier) @identifier)
            (jsx_text) @string
            (shorthand_property_identifier) @identifier
            (function_declaration
                name: (identifier) @identifier)
            (method_definition
                name: (property_identifier) @identifier)
            (class_declaration
                name: (identifier) @identifier)
                "#,
        extensions: &["js"],
    },
    LanguageSetting {
        type_: LanguageType::Typescript,
        name: "typescript",
        query: r#"
            (comment) @comment
            (string_fragment) @string
            (string) @string
            (variable_declarator
                name: (identifier) @identifier)
            (jsx_text) @string
            (shorthand_property_identifier) @identifier
            (function_declaration
                name: (identifier) @identifier)
            (method_definition
                name: (property_identifier) @identifier)
            (class_declaration
                name: (identifier) @identifier)
                "#,
        extensions: &["ts"],
    },
    LanguageSetting {
        type_: LanguageType::Html,
        name: "html",
        query: r#"
            (text) @string
            (comment) @comment
            (quoted_attribute_value) @string
            "#,
        extensions: &["html", "htm"],
    },
    LanguageSetting {
        type_: LanguageType::Css,
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
        type_: LanguageType::Go,
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
    pub type_: LanguageType,
    pub query: &'static str,
    pub name: &'static str,
    pub extensions: &'static [&'static str],
}

impl LanguageSetting {
    pub fn language(&self) -> Option<Language> {
        match self.type_ {
            LanguageType::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            LanguageType::Python => Some(tree_sitter_python::LANGUAGE.into()),
            LanguageType::Javascript => Some(tree_sitter_javascript::LANGUAGE.into()),
            LanguageType::Typescript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            LanguageType::Html => Some(tree_sitter_html::LANGUAGE.into()),
            LanguageType::Css => None,
            LanguageType::Go => Some(tree_sitter_go::LANGUAGE.into()),
            LanguageType::Text => None,
        }
    }
}

pub fn get_language_setting(language_type: LanguageType) -> Option<&'static LanguageSetting> {
    for setting in LANGUAGE_SETTINGS.iter() {
        if setting.type_ == language_type {
            if setting.language().is_some() {
                return Some(setting);
            }
        }
    }
    None
}

pub fn get_language_name_from_filename(filename: &str) -> Option<LanguageType> {
    let extension = filename.split('.').last().unwrap();
    for setting in LANGUAGE_SETTINGS.iter() {
        for ext in setting.extensions.iter() {
            if ext == &extension {
                return Some(setting.type_);
            }
        }
    }
    None
}

pub fn get_common_dictionary() -> impl Iterator<Item = &'static str> {
    COMMON_DICTIONARY.lines().filter(|l| !l.contains('#'))
}
