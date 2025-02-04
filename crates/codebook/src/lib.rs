pub mod dictionaries;
mod log;
pub mod parser;
pub mod queries;
mod splitter;

use std::sync::Arc;

use codebook_config::CodebookConfig;
use dictionaries::{dictionary, manager::DictionaryManager};
use dictionary::Dictionary;
use parser::WordLocation;

pub struct Codebook {
    config: Arc<CodebookConfig>,
    manager: DictionaryManager,
}

impl Codebook {
    pub fn new(config: Arc<CodebookConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        crate::log::init_logging();
        let manager = DictionaryManager::new(&config.cache_dir);
        Ok(Self { config, manager })
    }

    /// Get WordLocations for a block of text.
    /// Supply LanguageType, file path or both to use the correct code parser.
    pub fn spell_check(
        &self,
        text: &str,
        language: Option<queries::LanguageType>,
        file_path: Option<&str>,
    ) -> Vec<parser::WordLocation> {
        // get needed dictionary names
        // get needed dictionaries
        // call spell check on each dictionary
        let language = self.resolve_language(language, file_path);
        let dictionaries = self.get_dictionaries(Some(language));
        parser::find_locations(text, language, |word| {
            if self.config.is_allowed_word(word) {
                return true;
            }
            for dictionary in &dictionaries {
                if dictionary.check(word) {
                    return true;
                }
            }
            false
        })
    }

    fn resolve_language(
        &self,
        language_type: Option<queries::LanguageType>,
        path: Option<&str>,
    ) -> queries::LanguageType {
        // Check if we have a language_id first, fallback to path, fall back to text
        match language_type {
            Some(lang) => lang,
            None => match path {
                Some(path) => queries::get_language_name_from_filename(path),
                None => queries::LanguageType::Text,
            },
        }
    }

    fn get_dictionaries(
        &self,
        language: Option<queries::LanguageType>,
    ) -> Vec<Arc<dyn Dictionary>> {
        let mut dictionary_ids = self.config.get_dictionary_ids();
        match language {
            Some(lang) => {
                let language_dictionary_ids = lang.dictionary_ids();
                dictionary_ids.extend(language_dictionary_ids);
            }
            None => {}
        };
        // Push custom codebook dictionary. Could be removed later for a more general solution.
        dictionary_ids.push("codebook".to_string());
        let mut dictionaries = Vec::with_capacity(dictionary_ids.len());
        for dictionary_id in dictionary_ids {
            let dictionary = self.manager.get_dictionary(&dictionary_id);
            match dictionary {
                Some(d) => dictionaries.push(d),
                None => {}
            }
        }
        dictionaries
    }

    pub fn spell_check_file(&self, path: &str) -> Vec<WordLocation> {
        let lang_type = queries::get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path).unwrap();
        return self.spell_check(&file_text, Some(lang_type), None);
    }

    pub fn get_suggestions(&self, word: &str) -> Vec<String> {
        let dictionaries = self.get_dictionaries(None);
        let mut suggestions = Vec::new();
        for dict in dictionaries {
            suggestions.extend(dict.suggest(word));
        }
        suggestions.iter().take(5).cloned().collect()
    }
}
