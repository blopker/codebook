pub mod dictionary;
mod dictionary_repo;
pub mod downloader;
mod log;
pub mod parser;
pub mod queries;
mod splitter;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use codebook_config::CodebookConfig;
use dictionary::CodeDictionary;
use downloader::DictionaryDownloader;
use parser::WordLocation;

#[derive(Debug)]
pub struct Codebook {
    downloader: DictionaryDownloader,
    dictionary_cache: Arc<RwLock<HashMap<String, CodeDictionary>>>,
    config: Arc<CodebookConfig>,
}

impl Codebook {
    pub fn new(config: Arc<CodebookConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        crate::log::init_logging();
        let downloader = DictionaryDownloader::with_cache(&config.cache_dir);
        let files = downloader.get("en").unwrap();
        let mut dictionary_cache = HashMap::new();
        let dictionary = CodeDictionary::new(
            Arc::clone(&config),
            &files.aff_local_path,
            &files.dic_local_path,
        )?;
        dictionary_cache.insert("en".to_string(), dictionary);
        Ok(Self {
            downloader,
            dictionary_cache: Arc::new(RwLock::new(dictionary_cache)),
            config,
        })
    }

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
        // let dictionary_ids = self.config.get_settings().dictionaries;
        let dictionary_ids = vec!["en"];
        let dictionaries = self.get_dictionaries(&dictionary_ids, language);
        parser::find_locations(text, language, |word| {
            for dictionary in &dictionaries {
                if !dictionary.check(word) {
                    return false;
                }
            }
            true
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
        dictionary_ids: &[&str],
        language: queries::LanguageType,
    ) -> Vec<CodeDictionary> {
        let mut dictionaries = Vec::with_capacity(dictionary_ids.len());
        for dictionary_id in dictionary_ids {
            let dict = self.downloader.get(dictionary_id);
            if dict.is_ok() {
                let a = dict.unwrap();
                let d = CodeDictionary::new(
                    Arc::clone(&self.config),
                    &a.aff_local_path,
                    &a.dic_local_path,
                );
                if d.is_ok() {
                    dictionaries.push(d.unwrap());
                }
            }
        }
        println!("dic_ids: {:?}", dictionary_ids);
        println!("dic: {:?}", dictionaries);
        dictionaries
    }

    pub fn spell_check_file(&self, path: &str) -> Vec<WordLocation> {
        let lang_type = queries::get_language_name_from_filename(path);
        let file_text = std::fs::read_to_string(path).unwrap();
        return self.spell_check(&file_text, Some(lang_type), None);
    }

    pub fn get_suggestions(&self, word: &str) -> Vec<String> {
        self.dictionary_cache
            .read()
            .unwrap()
            .get("en")
            .unwrap()
            .suggest(word)
    }
}
