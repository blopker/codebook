use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use super::{
    dictionary::{self, TextDictionary},
    repo::{DictionaryRepo, HunspellRepo, TextRepo, get_repo},
};
use codebook_downloader::Downloader;
use dictionary::{Dictionary, HunspellDictionary};
use log::{debug, error};

pub struct DictionaryManager {
    dictionary_cache: Arc<RwLock<HashMap<String, Arc<dyn Dictionary>>>>,
    downloader: Downloader,
}

impl DictionaryManager {
    pub fn new(cache_dir: &PathBuf) -> Self {
        Self {
            dictionary_cache: Arc::new(RwLock::new(HashMap::new())),
            downloader: Downloader::new(cache_dir),
        }
    }

    pub fn get_dictionary(&self, id: &str) -> Option<Arc<dyn Dictionary>> {
        {
            let cache = self.dictionary_cache.read().unwrap();
            if let Some(dictionary) = cache.get(id) {
                return Some(dictionary.clone());
            }
        }
        let repo = match get_repo(id) {
            Some(r) => r,
            None => {
                debug!("Failed to get repo for dictionary, skipping: {id}");
                return None;
            }
        };

        let dictionary: Option<Arc<dyn Dictionary>> = match repo {
            DictionaryRepo::Hunspell(r) => self.get_hunspell_dictionary(r),
            DictionaryRepo::Text(r) => self.get_text_dictionary(r),
        };

        let mut cache = self.dictionary_cache.write().unwrap();
        match dictionary {
            Some(d) => {
                cache.insert(id.to_string(), d.clone());
                Some(d)
            }
            None => None,
        }
    }

    fn get_hunspell_dictionary(&self, repo: HunspellRepo) -> Option<Arc<dyn Dictionary>> {
        let aff_path = match self.downloader.get(&repo.aff_url) {
            Ok(path) => path,
            Err(e) => {
                error!("Error: {e:?}");
                return None;
            }
        };
        let dic_path = match self.downloader.get(&repo.dict_url) {
            Ok(path) => path,
            Err(e) => {
                error!("Error: {e:?}");
                return None;
            }
        };
        let dict =
            match HunspellDictionary::new(aff_path.to_str().unwrap(), dic_path.to_str().unwrap()) {
                Ok(dict) => dict,
                Err(e) => {
                    error!("Error: {e:?}");
                    return None;
                }
            };
        Some(Arc::new(dict))
    }

    fn get_text_dictionary(&self, repo: TextRepo) -> Option<Arc<dyn Dictionary>> {
        if let Some(text) = repo.text {
            return Some(Arc::new(TextDictionary::new(text)));
        }
        let text_path = match self.downloader.get(&repo.url.unwrap()) {
            Ok(path) => path,
            Err(e) => {
                error!("Error: {e:?}");
                return None;
            }
        };
        let dict = TextDictionary::new_from_path(&text_path);
        Some(Arc::new(dict))
    }
}
