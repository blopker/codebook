use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use super::{
    dictionary::{self, TextDictionary},
    repo::{DictionaryRepo, HunspellRepo, TextRepo, get_repo},
    transliteration::TransliteratingDictionary,
};
use codebook_downloader::{Downloader, PermanentHttpError};
use dictionary::{Dictionary, HunspellDictionary};
use log::{debug, error, warn};

/// How long to wait before retrying a dictionary that failed to load for a
/// transient reason (network down, server error). Loading involves blocking
/// HTTP with retries and backoff; without this cooldown an offline first run
/// would re-attempt the download on every spell check — every keystroke in
/// the LSP.
const TRANSIENT_FAILURE_COOLDOWN: Duration = Duration::from_secs(5 * 60);

/// A dictionary load failure. Permanent failures (the server definitively
/// answered 4xx) are not retried for the life of the process; transient ones
/// retry after a cooldown.
struct LoadError {
    permanent: bool,
}

pub struct DictionaryManager {
    dictionary_cache: RwLock<HashMap<String, Arc<dyn Dictionary>>>,
    failed_loads: RwLock<HashMap<String, (Instant, Duration)>>,
    downloader: Downloader,
    local_dir: Option<PathBuf>,
}

impl DictionaryManager {
    pub fn new(cache_dir: &PathBuf) -> Self {
        Self::with_local_dir(cache_dir, None)
    }

    /// Create a manager that resolves dictionaries from a local directory of
    /// `{id}.txt` word lists or `{id}.aff` + `{id}.dic` Hunspell pairs before
    /// falling back to the download repo. Tests use this with checked-in
    /// fixtures so `cargo test` never touches the network.
    pub fn with_local_dir(cache_dir: &PathBuf, local_dir: Option<PathBuf>) -> Self {
        Self {
            dictionary_cache: RwLock::new(HashMap::new()),
            failed_loads: RwLock::new(HashMap::new()),
            downloader: Downloader::new(cache_dir),
            local_dir,
        }
    }

    pub fn get_dictionary(&self, id: &str) -> Option<Arc<dyn Dictionary>> {
        {
            let cache = self.dictionary_cache.read().unwrap();
            if let Some(dictionary) = cache.get(id) {
                return Some(dictionary.clone());
            }
        }

        if let Some(d) = self.get_local_dictionary(id) {
            self.dictionary_cache
                .write()
                .unwrap()
                .insert(id.to_string(), d.clone());
            return Some(d);
        }

        let repo = match get_repo(id) {
            Some(r) => r,
            None => {
                debug!("Failed to get repo for dictionary, skipping: {id}");
                return None;
            }
        };

        {
            let failed = self.failed_loads.read().unwrap();
            if let Some((failed_at, cooldown)) = failed.get(id)
                && failed_at.elapsed() < *cooldown
            {
                debug!("Dictionary '{id}' recently failed to load, skipping until cooldown");
                return None;
            }
        }

        let dictionary: Result<Arc<dyn Dictionary>, LoadError> = match repo {
            DictionaryRepo::Hunspell(r) => self.get_hunspell_dictionary(r),
            DictionaryRepo::Text(r) => self.get_text_dictionary(r),
        };

        match dictionary {
            Ok(d) => {
                let mut cache = self.dictionary_cache.write().unwrap();
                cache.insert(id.to_string(), d.clone());
                self.failed_loads.write().unwrap().remove(id);
                Some(d)
            }
            Err(e) => {
                let cooldown = if e.permanent {
                    warn!("Dictionary '{id}' does not exist upstream, not retrying");
                    Duration::MAX
                } else {
                    warn!(
                        "Failed to load dictionary '{id}', will not retry for {}s",
                        TRANSIENT_FAILURE_COOLDOWN.as_secs()
                    );
                    TRANSIENT_FAILURE_COOLDOWN
                };
                self.failed_loads
                    .write()
                    .unwrap()
                    .insert(id.to_string(), (Instant::now(), cooldown));
                None
            }
        }
    }

    /// Load a dictionary from the local override directory, if configured.
    fn get_local_dictionary(&self, id: &str) -> Option<Arc<dyn Dictionary>> {
        let dir = self.local_dir.as_ref()?;
        let txt = dir.join(format!("{id}.txt"));
        if txt.is_file() {
            return Some(Arc::new(TextDictionary::new_from_path(&txt)));
        }
        let aff = dir.join(format!("{id}.aff"));
        let dic = dir.join(format!("{id}.dic"));
        if aff.is_file() && dic.is_file() {
            match HunspellDictionary::new(aff.to_str()?, dic.to_str()?) {
                Ok(dict) => return Some(Arc::new(dict)),
                Err(e) => error!("Failed to load local dictionary '{id}': {e:?}"),
            }
        }
        None
    }

    fn download(&self, url: &str) -> Result<PathBuf, LoadError> {
        self.downloader.get(url).map_err(|e| {
            error!("Error: {e:?}");
            LoadError {
                permanent: e.downcast_ref::<PermanentHttpError>().is_some(),
            }
        })
    }

    fn get_hunspell_dictionary(
        &self,
        repo: HunspellRepo,
    ) -> Result<Arc<dyn Dictionary>, LoadError> {
        let aff_path = self.download(&repo.aff_url)?;
        let dic_path = self.download(&repo.dict_url)?;
        let dict =
            match HunspellDictionary::new(aff_path.to_str().unwrap(), dic_path.to_str().unwrap()) {
                Ok(dict) => dict,
                Err(e) => {
                    error!("Error: {e:?}");
                    return Err(LoadError { permanent: false });
                }
            };
        let base: Arc<dyn Dictionary> = Arc::new(dict);
        Ok(match repo.transliteration {
            Some(t) => Arc::new(TransliteratingDictionary::new(base, t.variants_fn())),
            None => base,
        })
    }

    fn get_text_dictionary(&self, repo: TextRepo) -> Result<Arc<dyn Dictionary>, LoadError> {
        if let Some(text) = repo.text {
            return Ok(Arc::new(TextDictionary::new(text)));
        }
        let text_path = self.download(&repo.url.unwrap())?;
        let dict = TextDictionary::new_from_path(&text_path);
        Ok(Arc::new(dict))
    }
}
