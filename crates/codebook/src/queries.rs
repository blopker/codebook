use std::str::FromStr;

use tree_sitter::Language;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum LanguageType {
    Bash,
    C,
    CSharp,
    Cpp,
    Css,
    Elixir,
    Erlang,
    Go,
    HTML,
    Haskell,
    Java,
    Javascript,
    Latex,
    Lua,
    Markdown,
    Odin,
    Php,
    Python,
    R,
    Ruby,
    Rust,
    Swift,
    TOML,
    Text,
    Typescript,
    Tsx,
    Typst,
    VHDL,
    YAML,
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
            for ext in language.extensions.iter() {
                if s == *ext {
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
        extensions: &["cpp", "cc", "cxx", "hpp", "hh", "hxx", "cppm", "ixx", "mxx"],
    },
    LanguageSetting {
        type_: LanguageType::Elixir,
        ids: &["elixir"],
        dictionary_ids: &["elixir"],
        query: include_str!("queries/elixir.scm"),
        extensions: &["ex", "exs"],
    },
    LanguageSetting {
        type_: LanguageType::Erlang,
        ids: &["erlang"],
        dictionary_ids: &["erlang"],
        query: include_str!("queries/erlang.scm"),
        extensions: &["erl", "hrl"],
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
        ids: &["javascript", "javascriptreact", "jsx"],
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
        ids: &["typescript"],
        dictionary_ids: &["typescript"],
        query: include_str!("queries/typescript.scm"),
        extensions: &["ts"],
    },
    LanguageSetting {
        type_: LanguageType::Tsx,
        ids: &["typescriptreact", "tsx"],
        dictionary_ids: &["typescriptreact"],
        query: include_str!("queries/typescript.scm"),
        extensions: &["tsx"],
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
        ids: &["html", "vue", "vue.js", "astro"],
        dictionary_ids: &["html"],
        query: include_str!("queries/html.scm"),
        extensions: &["html", "htm", "vue", "astro"],
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
        type_: LanguageType::Swift,
        ids: &["swift"],
        dictionary_ids: &["swift"],
        query: include_str!("queries/swift.scm"),
        extensions: &["swift"],
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
        type_: LanguageType::Markdown,
        ids: &["markdown"],
        dictionary_ids: &[],
        query: include_str!("queries/markdown.scm"),
        extensions: &["md", "markdown"],
    },
    LanguageSetting {
        type_: LanguageType::Bash,
        ids: &[
            "bash",
            "shellscript",
            "sh",
            "shell script",
            "shell",
            "zsh",
            "fish",
        ],
        dictionary_ids: &["bash"],
        query: include_str!("queries/bash.scm"),
        extensions: &["sh", "bash"],
    },
    LanguageSetting {
        type_: LanguageType::Odin,
        ids: &["odin"],
        dictionary_ids: &["odin"],
        query: include_str!("queries/odin.scm"),
        extensions: &["odin"],
    },
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
        type_: LanguageType::YAML,
        ids: &["yaml", "yml"],
        dictionary_ids: &["yaml"],
        query: include_str!("queries/yaml.scm"),
        extensions: &["yaml", "yml"],
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
    LanguageSetting {
        type_: LanguageType::VHDL,
        ids: &["vhdl"],
        dictionary_ids: &["vhdl"],
        query: include_str!("queries/vhdl.scm"),
        extensions: &["vhd", "vhdl"],
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
            LanguageType::Erlang => Some(tree_sitter_erlang::LANGUAGE.into()),
            LanguageType::Go => Some(tree_sitter_go::LANGUAGE.into()),
            LanguageType::HTML => Some(tree_sitter_html::LANGUAGE.into()),
            LanguageType::Haskell => Some(tree_sitter_haskell::LANGUAGE.into()),
            LanguageType::Java => Some(tree_sitter_java::LANGUAGE.into()),
            LanguageType::Javascript => Some(tree_sitter_javascript::LANGUAGE.into()),
            LanguageType::Latex => Some(codebook_tree_sitter_latex::LANGUAGE.into()),
            LanguageType::Lua => Some(tree_sitter_lua::LANGUAGE.into()),
            LanguageType::Markdown => Some(tree_sitter_md::LANGUAGE.into()),
            LanguageType::Odin => Some(tree_sitter_odin_codebook::LANGUAGE.into()),
            LanguageType::Php => Some(tree_sitter_php::LANGUAGE_PHP.into()),
            LanguageType::Python => Some(tree_sitter_python::LANGUAGE.into()),
            LanguageType::R => Some(tree_sitter_r::LANGUAGE.into()),
            LanguageType::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
            LanguageType::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            LanguageType::Swift => Some(tree_sitter_swift::LANGUAGE.into()),
            LanguageType::TOML => Some(tree_sitter_toml_ng::LANGUAGE.into()),
            LanguageType::Text => None,
            LanguageType::Typescript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            LanguageType::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            LanguageType::Typst => Some(codebook_tree_sitter_typst::LANGUAGE.into()),
            LanguageType::VHDL => Some(tree_sitter_vhdl::LANGUAGE.into()),
            LanguageType::YAML => Some(tree_sitter_yaml::LANGUAGE.into()),
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

    /// Allowed full capture names. Any capture in a .scm file must be one of these.
    /// The special "language" tag is used internally (e.g., ruby heredocs) and is
    /// not exposed for user filtering.
    const ALLOWED_TAGS: &[&str] = &[
        "comment",
        "comment.line",
        "comment.block",
        "string",
        "string.special",
        "string.heredoc",
        "identifier",
        "identifier.function",
        "identifier.type",
        "identifier.parameter",
        "identifier.field",
        "identifier.variable",
        "identifier.constant",
        "identifier.module",
        "language",
    ];

    #[test]
    fn test_all_capture_names_use_allowed_tags() {
        for language_setting in LANGUAGE_SETTINGS {
            if language_setting.type_ == LanguageType::Text {
                continue;
            }

            let language = language_setting.language().unwrap_or_else(|| {
                panic!("Failed to get language for {:?}", language_setting.type_)
            });

            let query = Query::new(&language, language_setting.query).unwrap_or_else(|e| {
                panic!(
                    "Invalid query for language {:?}: {:?}",
                    language_setting.type_, e
                )
            });

            for name in query.capture_names() {
                let is_allowed = ALLOWED_TAGS.contains(name) || name.starts_with("injection.");
                assert!(
                    is_allowed,
                    "Language {:?} uses unknown capture tag @{name}. \
                     Allowed tags: {ALLOWED_TAGS:?} (plus injection.* tags)",
                    language_setting.type_,
                );
            }
        }
    }

    #[test]
    fn test_no_overlap_in_ids_and_extensions() {
        use std::collections::HashMap;

        // Map every id and extension to the language that owns it
        let mut seen: HashMap<&str, LanguageType> = HashMap::new();

        for setting in LANGUAGE_SETTINGS {
            for &id in setting.ids {
                if let Some(&prev) = seen.get(id) {
                    panic!(
                        "Duplicate id/extension {id:?}: used by both {:?} and {:?}",
                        prev, setting.type_
                    );
                }
                seen.insert(id, setting.type_);
            }
            for &ext in setting.extensions {
                if let Some(&prev) = seen.get(ext) {
                    // Allow overlap within the same language (e.g. "hs" in both ids and extensions)
                    if prev != setting.type_ {
                        panic!(
                            "Duplicate id/extension {ext:?}: used by both {:?} and {:?}",
                            prev, setting.type_
                        );
                    }
                }
                seen.insert(ext, setting.type_);
            }
        }
    }
}
