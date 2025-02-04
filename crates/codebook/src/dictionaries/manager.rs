use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::{
    dictionary::{self, TextDictionary},
    repo::{get_repo, DictionaryRepo, HunspellRepo, TextRepo},
};
use codebook_config::CodebookConfig;
use dictionary::{Dictionary, HunspellDictionary};
use downloader::Downloader;
use log::info;

pub struct DictionaryManager {
    config: Arc<CodebookConfig>,
    dictionary_cache: Arc<RwLock<HashMap<String, Arc<dyn Dictionary>>>>,
    downloader: Downloader,
}

impl DictionaryManager {
    pub fn new(config: Arc<CodebookConfig>) -> Self {
        let cache_path = config.cache_dir.clone();
        Self {
            config,
            dictionary_cache: Arc::new(RwLock::new(HashMap::new())),
            downloader: Downloader::new(cache_path).unwrap(),
        }
    }

    pub fn get_dictionary(&self, id: &str) -> Option<Arc<dyn Dictionary>> {
        let mut cache = self.dictionary_cache.write().unwrap();
        if let Some(dictionary) = cache.get(id) {
            return Some(dictionary.clone());
        }
        let repo = match get_repo(id) {
            Some(r) => r,
            None => {
                info!("Failed to get repo for dictionary: {}", id);
                return None;
            }
        };

        let dictionary: Option<Arc<dyn Dictionary>> = match repo {
            DictionaryRepo::Hunspell(r) => self.get_hunspell_dictionary(r),
            DictionaryRepo::Text(r) => self.get_text_dictionary(r),
        };

        match dictionary {
            Some(d) => {
                cache.insert(id.to_string(), d);
                Some(cache.get(id).unwrap().clone())
            }
            None => None,
        }
    }

    fn get_hunspell_dictionary(&self, repo: HunspellRepo) -> Option<Arc<dyn Dictionary>> {
        let aff_path = match self.downloader.get(&repo.aff_url) {
            Ok(path) => path,
            Err(e) => {
                info!("Error: {:?}", e);
                return None;
            }
        };
        let dic_path = match self.downloader.get(&repo.dict_url) {
            Ok(path) => path,
            Err(e) => {
                info!("Error: {:?}", e);
                return None;
            }
        };
        let dict =
            match HunspellDictionary::new(aff_path.to_str().unwrap(), dic_path.to_str().unwrap()) {
                Ok(dict) => dict,
                Err(e) => {
                    info!("Error: {:?}", e);
                    return None;
                }
            };
        Some(Arc::new(dict))
    }

    fn get_text_dictionary(&self, repo: TextRepo) -> Option<Arc<dyn Dictionary>> {
        if repo.text.is_some() {
            return Some(Arc::new(TextDictionary::new(repo.text.unwrap())));
        }
        let text_path = match self.downloader.get(&repo.url.unwrap()) {
            Ok(path) => path,
            Err(e) => {
                info!("Error: {:?}", e);
                return None;
            }
        };
        let dict = TextDictionary::new(text_path.to_str().unwrap());
        Some(Arc::new(dict))
    }
}
