use std::str::FromStr;

use tree_sitter::Language;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum LanguageType {
    Bash,
    C,
    CSharp,
    Cpp,
    Css,
    Elixir,
    Go,
    HTML,
    Haskell,
    Java,
    Javascript,
    Latex,
    Lua,
    Php,
    Python,
    R,
    Ruby,
    Rust,
    TOML,
    Text,
    Typescript,
    Typst,
    Zig,
}

impl FromStr for LanguageType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for language in LANGUAGE_SETTINGS.iter() {
            for id in language.ids.iter() {
                if s == *id {
                    return Ok(language.type_);
                }
            }
        }
        Ok(LanguageType::Text)
    }
}

impl LanguageType {
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

// Language ids documented at https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentItem
pub static LANGUAGE_SETTINGS: &[LanguageSetting] = &[
    LanguageSetting {
        type_: LanguageType::Rust,
        ids: &["rust"],
        dictionary_ids: &["rust"],
        query: include_str!("queries/rust.scm"),
        extensions: &["rs"],
    },
    LanguageSetting {
        type_: LanguageType::C,
        ids: &["c"],
        dictionary_ids: &["c"],
        query: include_str!("queries/c.scm"),
        extensions: &["c", "h"],
    },
    LanguageSetting {
        type_: LanguageType::Cpp,
        ids: &["cpp", "c++"],
        dictionary_ids: &["cpp"],
        query: include_str!("queries/cpp.scm"),
        extensions: &["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
    },
    LanguageSetting {
        type_: LanguageType::Elixir,
        ids: &["elixir"],
        dictionary_ids: &["elixir"],
        query: include_str!("queries/elixir.scm"),
        extensions: &["ex", "exs"],
    },
    LanguageSetting {
        type_: LanguageType::Python,
        ids: &["python"],
        dictionary_ids: &["python"],
        query: include_str!("queries/python.scm"),
        extensions: &["py"],
    },
    LanguageSetting {
        type_: LanguageType::Java,
        ids: &["java"],
        dictionary_ids: &["java"],
        query: include_str!("queries/java.scm"),
        extensions: &["java"],
    },
    LanguageSetting {
        type_: LanguageType::Javascript,
        ids: &["javascript", "javascriptreact"],
        dictionary_ids: &["javascript", "javascriptreact"],
        query: include_str!("queries/javascript.scm"),
        extensions: &["js", "jsx"],
    },
    LanguageSetting {
        type_: LanguageType::Latex,
        ids: &["latex"],
        dictionary_ids: &["latex"],
        query: include_str!("queries/latex.scm"),
        extensions: &["tex", "latex", "ltx"],
    },
    LanguageSetting {
        type_: LanguageType::Typescript,
        ids: &["typescript", "typescriptreact"],
        dictionary_ids: &["typescript", "typescriptreact"],
        query: include_str!("queries/typescript.scm"),
        extensions: &["ts", "tsx"],
    },
    LanguageSetting {
        type_: LanguageType::Haskell,
        ids: &["hs"],
        dictionary_ids: &["haskell"],
        query: include_str!("queries/haskell.scm"),
        extensions: &["hs"],
    },
    LanguageSetting {
        type_: LanguageType::HTML,
        ids: &["html"],
        dictionary_ids: &["html"],
        query: include_str!("queries/html.scm"),
        extensions: &["html", "htm"],
    },
    LanguageSetting {
        type_: LanguageType::Css,
        ids: &["css"],
        dictionary_ids: &["css"],
        query: include_str!("queries/css.scm"),
        extensions: &["css"],
    },
    LanguageSetting {
        type_: LanguageType::Go,
        ids: &["go"],
        dictionary_ids: &["go"],
        query: include_str!("queries/go.scm"),
        extensions: &["go"],
    },
    LanguageSetting {
        type_: LanguageType::TOML,
        ids: &["toml"],
        dictionary_ids: &["toml"],
        query: include_str!("queries/toml.scm"),
        extensions: &["toml"],
    },
    LanguageSetting {
        type_: LanguageType::Ruby,
        ids: &["ruby"],
        dictionary_ids: &["ruby"],
        query: include_str!("queries/ruby.scm"),
        extensions: &["rb"],
    },
    LanguageSetting {
        type_: LanguageType::Lua,
        ids: &["lua"],
        dictionary_ids: &["lua"],
        query: include_str!("queries/lua.scm"),
        extensions: &["lua"],
    },
    LanguageSetting {
        type_: LanguageType::Bash,
        ids: &["bash", "shellscript", "sh", "shell script"],
        dictionary_ids: &["bash"],
        query: include_str!("queries/bash.scm"),
        extensions: &["sh", "bash"],
    },
    // Added PHP
    LanguageSetting {
        type_: LanguageType::Php,
        ids: &["php"],
        dictionary_ids: &["php"],
        query: include_str!("queries/php.scm"),
        extensions: &["php"],
    },
    LanguageSetting {
        type_: LanguageType::R,
        ids: &["r"],
        dictionary_ids: &["r"],
        query: include_str!("queries/r.scm"),
        extensions: &["r", "R"],
    },
    LanguageSetting {
        type_: LanguageType::Zig,
        ids: &["zig"],
        dictionary_ids: &["zig"],
        query: include_str!("queries/zig.scm"),
        extensions: &["zig"],
    },
    LanguageSetting {
        type_: LanguageType::CSharp,
        ids: &["csharp"],
        dictionary_ids: &["csharp"],
        query: include_str!("queries/csharp.scm"),
        extensions: &["cs"],
    },
    LanguageSetting {
        type_: LanguageType::Typst,
        ids: &["typst"],
        dictionary_ids: &["typst"],
        query: include_str!("queries/typst.scm"),
        extensions: &["typ"],
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
            LanguageType::Bash => Some(tree_sitter_bash::LANGUAGE.into()),
            LanguageType::C => Some(tree_sitter_c::LANGUAGE.into()),
            LanguageType::CSharp => Some(tree_sitter_c_sharp::LANGUAGE.into()),
            LanguageType::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            LanguageType::Css => Some(tree_sitter_css::LANGUAGE.into()),
            LanguageType::Elixir => Some(tree_sitter_elixir::LANGUAGE.into()),
            LanguageType::Go => Some(tree_sitter_go::LANGUAGE.into()),
            LanguageType::HTML => Some(tree_sitter_html::LANGUAGE.into()),
            LanguageType::Haskell => Some(tree_sitter_haskell::LANGUAGE.into()),
            LanguageType::Java => Some(tree_sitter_java::LANGUAGE.into()),
            LanguageType::Javascript => Some(tree_sitter_javascript::LANGUAGE.into()),
            LanguageType::Latex => Some(codebook_tree_sitter_latex::language()),
            LanguageType::Lua => Some(tree_sitter_lua::LANGUAGE.into()),
            LanguageType::Php => Some(tree_sitter_php::LANGUAGE_PHP.into()),
            LanguageType::Python => Some(tree_sitter_python::LANGUAGE.into()),
            LanguageType::R => Some(tree_sitter_r::LANGUAGE.into()),
            LanguageType::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
            LanguageType::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            LanguageType::TOML => Some(tree_sitter_toml_ng::LANGUAGE.into()),
            LanguageType::Text => None,
            LanguageType::Typescript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            LanguageType::Typst => Some(codebook_tree_sitter_typst::language()),
            LanguageType::Zig => Some(tree_sitter_zig::LANGUAGE.into()),
        }
    }
}

pub fn get_language_setting(language_type: LanguageType) -> Option<&'static LanguageSetting> {
    LANGUAGE_SETTINGS
        .iter()
        .find(|&setting| setting.type_ == language_type && setting.language().is_some())
}

pub fn get_language_name_from_filename(filename: &str) -> LanguageType {
    let extension = filename.split('.').next_back().unwrap();
    for setting in LANGUAGE_SETTINGS {
        for ext in setting.extensions {
            if ext == &extension {
                return setting.type_;
            }
        }
    }
    LanguageType::Text
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Query;

    #[test]
    fn test_all_queries_are_valid() {
        for language_setting in LANGUAGE_SETTINGS {
            // Skip testing Text since it doesn't have a language or query
            if language_setting.type_ == LanguageType::Text {
                continue;
            }

            // Get the language for this setting
            let language = match language_setting.language() {
                Some(lang) => lang,
                None => {
                    panic!("Failed to get language for {:?}", language_setting.type_);
                }
            };

            // Try to create a Query with the language and query
            let query_result = Query::new(&language, language_setting.query);

            // Assert that the query is valid
            assert!(
                query_result.is_ok(),
                "Invalid query for language {:?}: {:?}",
                language_setting.type_,
                query_result.err()
            );
        }
    }
}
