pub mod dictionary;
pub mod downloader;
mod queries;
mod splitter;

use std::sync::Arc;

use codebook_config::CodebookConfig;
use dictionary::CodeDictionary;
use downloader::DictionaryDownloader;

#[derive(Debug)]
pub struct Codebook {
    downloader: DictionaryDownloader,
    pub dictionary: CodeDictionary,
    config: Arc<CodebookConfig>,
}

impl Codebook {
    pub fn new(config: Arc<CodebookConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let downloader = DictionaryDownloader::with_cache(&config.cache_dir);
        let files = downloader.get("en").unwrap();
        let dictionary = CodeDictionary::new(
            Arc::clone(&config),
            &files.aff_local_path,
            &files.dic_local_path,
        )?;
        Ok(Self {
            downloader,
            dictionary,
            config,
        })
    }
}
