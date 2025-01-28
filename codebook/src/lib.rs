pub mod dictionary;
mod dictionary_repo;
pub mod downloader;
mod log;
pub mod queries;
mod splitter;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use codebook_config::CodebookConfig;
use dictionary::CodeDictionary;
use downloader::DictionaryDownloader;

#[derive(Debug)]
pub struct Codebook {
    // downloader: DictionaryDownloader,
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
            // downloader,
            dictionary_cache: Arc::new(RwLock::new(dictionary_cache)),
            config,
        })
    }

    pub fn spell_check(
        &self,
        text: &str,
        language: Option<queries::LanguageType>,
        file_path: Option<&str>,
    ) -> Vec<dictionary::SpellCheckResult> {
        self.dictionary_cache
            .read()
            .unwrap()
            .get("en")
            .unwrap()
            .spell_check(text, language, file_path)
    }

    pub fn spell_check_file(&self, file_path: &str) -> Vec<dictionary::SpellCheckResult> {
        self.dictionary_cache
            .read()
            .unwrap()
            .get("en")
            .unwrap()
            .spell_check_file(file_path)
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
