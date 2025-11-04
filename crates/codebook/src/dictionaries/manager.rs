use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
};

use crate::dictionaries::repo::TextRepoLocation;

use super::{
    dictionary::{self, TextDictionary},
    repo::{DictionaryRepo, HunspellRepo, TextRepo, get_repo},
};
use codebook_config::CustomDictionariesEntry;
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
            downloader: Downloader::new(cache_dir).unwrap(),
        }
    }

    pub fn invalidate_cache_entry(&self, id: &str) {
        let mut cache = self.dictionary_cache.write().unwrap();
        cache.remove(id);
    }

    pub fn get_dictionary(
        &self,
        id: &str,
        custom_dicts_defs: &[CustomDictionariesEntry],
    ) -> Option<Arc<dyn Dictionary>> {
        {
            let cache = self.dictionary_cache.read().unwrap();
            if let Some(dictionary) = cache.get(id) {
                return Some(dictionary.clone());
            }
        }

        let repo = if let Some(custom_dict) = custom_dicts_defs.iter().find(|d| d.name == id) {
            DictionaryRepo::Text(TextRepo {
                name: custom_dict.name.clone(),
                text_location: TextRepoLocation::LocalFile(custom_dict.path.clone()),
            })
        } else {
            let repo = get_repo(id);
            if repo.is_none() {
                debug!("Failed to get repo for dictionary, skipping: {id}");
            }
            repo?
        };

        let dictionary: Option<Arc<dyn Dictionary>> = match repo {
            DictionaryRepo::Hunspell(r) => self.get_hunspell_dictionary(r),
            DictionaryRepo::Text(r) => self.get_text_dictionary(r),
        };

        let mut cache = self.dictionary_cache.write().unwrap();

        if let Some(dictionary) = &dictionary {
            cache.insert(id.to_string(), dictionary.clone());
        }

        dictionary
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
        const FAILED_TO_READ_DICT_ERR: &'static str = "Failed to read dictionary file";

        let dict = match repo.text_location {
            super::repo::TextRepoLocation::Url(url) => {
                let text_path = self
                    .downloader
                    .get(&url)
                    .inspect_err(|e| error!("Error: {e:?}"))
                    .ok()?;

                TextDictionary::try_from(&text_path)
                    .inspect_err(|_| error!("{}: {}", FAILED_TO_READ_DICT_ERR, text_path.display()))
                    .ok()?
            }
            super::repo::TextRepoLocation::LocalFile(path) => {
                let text_path = PathBuf::from_str(&path)
                    .inspect_err(|e| error!("Error: {e:?}"))
                    .ok()?;

                TextDictionary::try_from(&text_path)
                    .inspect_err(|_| error!("{}: {}", FAILED_TO_READ_DICT_ERR, text_path.display()))
                    .ok()?
            }
            super::repo::TextRepoLocation::Text(text) => TextDictionary::new(text),
        };

        Some(Arc::new(dict))
    }
}
