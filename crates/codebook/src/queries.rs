use tree_sitter::Language;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LanguageType {
    Css,
    C,
    Go,
    Html,
    Javascript,
    Python,
    Ruby,
    Rust,
    TOML,
    Text,
    Typescript,
}

impl LanguageType {
    pub fn from_str(s: &str) -> LanguageType {
        for language in LANGUAGE_SETTINGS.iter() {
            for id in language.ids.iter() {
                if s == *id {
                    return language.type_;
                }
            }
        }
        LanguageType::Text
    }
    pub fn dictionary_ids(&self) -> Vec<String> {
        for language in LANGUAGE_SETTINGS.iter() {
            if self == &language.type_ {
                return language
                    .dictionary_ids
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
        }
        vec![]
    }
}

// Use https://intmainreturn0.com/ts-visualizer/ to help with writing grammar queries
pub static LANGUAGE_SETTINGS: &[LanguageSetting] = &[
    LanguageSetting {
        type_: LanguageType::Rust,
        ids: &["rust"],
        dictionary_ids: &["rust"],
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
        type_: LanguageType::C,
        ids: &["c"],
        dictionary_ids: &["c"],
        query: r#"
                (comment) @comment
                (preproc_def
                name: (identifier) @identifier)
                (type_definition
                declarator: (type_identifier) @identifier)
                (struct_specifier
                name: (type_identifier) @identifier)
                (field_declaration
                    declarator: (field_identifier) @identifier)
                (pointer_declarator
                    declarator: (field_identifier) @identifier)
                (enum_specifier
                name: (type_identifier) @identifier)
                (enumerator
                    name: (identifier) @identifier)
                (init_declarator
                    declarator: (identifier) @identifier)
                (pointer_declarator
                    declarator: (identifier) @identifier)
                (init_declarator
                    (string_literal
                    (string_content) @string_content))
                (function_declarator
                    declarator: (identifier) @identifier)
                (parameter_declaration
                    declarator: (identifier) @identifier)
                    (call_expression
                (argument_list
                    (string_literal
                        [(string_content) (escape_sequence)] @string)))
                "#,
        extensions: &["c", "h"],
    },
    LanguageSetting {
        type_: LanguageType::Python,
        ids: &["python"],
        dictionary_ids: &["python"],
        query: r#"
            (comment) @comment
            (string) @string
            (function_definition
                name: (identifier) @identifier)
            (function_definition
                parameters: (parameters) @identifier)
            (class_definition
                name: (identifier) @identifier)
            (assignment
                (identifier) @identifier)
                "#,
        extensions: &["py"],
    },
    LanguageSetting {
        type_: LanguageType::Javascript,
        ids: &["javascript", "javascriptreact"],
        dictionary_ids: &["javascript", "javascriptreact"],
        query: r#"
            (comment) @comment
            (string_fragment) @string
            (variable_declarator
                name: (identifier) @identifier)
            (jsx_text) @string
            (shorthand_property_identifier) @identifier
            (function_declaration
                name: (identifier) @identifier)
            (function_declaration
                parameters: (formal_parameters
                (identifier) @identifier))
            (method_definition
                name: (property_identifier) @identifier)
            (class_declaration
                name: (identifier) @identifier)
                "#,
        extensions: &["js", "jsx"],
    },
    LanguageSetting {
        type_: LanguageType::Typescript,
        ids: &["typescript", "typescriptreact"],
        dictionary_ids: &["typescript", "typescriptreact"],
        query: r#"
            (comment) @comment
            (string_fragment) @string
            (variable_declarator
                name: (identifier) @identifier)
            (jsx_text) @string
            (shorthand_property_identifier) @identifier
            (function_declaration
                name: (identifier) @identifier)
            (formal_parameters
                (required_parameter
                pattern: (identifier) @identifier))
            (formal_parameters
                (optional_parameter
                pattern: (identifier) @identifier))
            (method_definition
                name: (property_identifier) @identifier)
            (class_declaration
                name: (type_identifier) @identifier)
            (public_field_definition
                name: (property_identifier) @identifier)
                "#,
        extensions: &["ts", "tsx"],
    },
    LanguageSetting {
        type_: LanguageType::Html,
        ids: &["html"],
        dictionary_ids: &["html"],
        query: r#"
            (text) @string
            (comment) @comment
            (quoted_attribute_value) @string
            "#,
        extensions: &["html", "htm"],
    },
    LanguageSetting {
        type_: LanguageType::Css,
        ids: &["css"],
        dictionary_ids: &["css"],
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
        ids: &["go"],
        dictionary_ids: &["go"],
        query: r#"
                (comment) @comment
                (argument_list (interpreted_string_literal) @string)
                (function_declaration (identifier) @identifier)
                (raw_string_literal) @raw_string
                (expression_list
                    (interpreted_string_literal) @string
                )
                (var_spec (identifier) @identifier)
                "#,
        extensions: &["go"],
    },
    LanguageSetting {
        type_: LanguageType::TOML,
        ids: &["toml"],
        dictionary_ids: &["toml"],
        query: r#"
            (string) @string
            (comment) @comment
            "#,
        extensions: &["toml"],
    },
    LanguageSetting {
        type_: LanguageType::Ruby,
        ids: &["ruby"],
        dictionary_ids: &["ruby"],
        query: r#"
            (string) @string
            (comment) @comment
            (assignment (identifier) @identifier)
            (method
              (method_parameters (keyword_parameter (identifier) @identifier)))
            (method
              (method_parameters (identifier) @identifier))
            (heredoc_body
              (heredoc_content) @string
              (heredoc_end) @language
              (#downcase! @language))
            "#,
        extensions: &["rb"],
    },
];

#[derive(Debug)]
pub struct LanguageSetting {
    pub type_: LanguageType,
    pub query: &'static str,
    /// ID from https://code.visualstudio.com/docs/languages/identifiers
    pub ids: &'static [&'static str],
    pub dictionary_ids: &'static [&'static str],
    pub extensions: &'static [&'static str],
}

impl LanguageSetting {
    pub fn language(&self) -> Option<Language> {
        match self.type_ {
            LanguageType::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            LanguageType::C => Some(tree_sitter_c::LANGUAGE.into()),
            LanguageType::Python => Some(tree_sitter_python::LANGUAGE.into()),
            LanguageType::Javascript => Some(tree_sitter_javascript::LANGUAGE.into()),
            LanguageType::Typescript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            LanguageType::Html => Some(tree_sitter_html::LANGUAGE.into()),
            LanguageType::Css => Some(tree_sitter_css::LANGUAGE.into()),
            LanguageType::Go => Some(tree_sitter_go::LANGUAGE.into()),
            LanguageType::TOML => Some(tree_sitter_toml_ng::LANGUAGE.into()),
            LanguageType::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
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

pub fn get_language_name_from_filename(filename: &str) -> LanguageType {
    let extension = filename.split('.').last().unwrap();
    for setting in LANGUAGE_SETTINGS.iter() {
        for ext in setting.extensions.iter() {
            if ext == &extension {
                return setting.type_;
            }
        }
    }
    LanguageType::Text
}
