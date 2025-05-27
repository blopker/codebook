use std::{
    num::NonZero,
    sync::{Arc, RwLock},
};

use lru::LruCache;
use tower_lsp::lsp_types::{TextDocumentItem, Url};

#[derive(Debug, Clone)]
pub struct TextDocumentCacheItem {
    pub text: String,
    pub uri: Url,
    pub version: Option<i32>,
    pub language_id: Option<String>,
}

impl TextDocumentCacheItem {
    pub fn new(
        uri: &Url,
        version: Option<i32>,
        language_id: Option<&str>,
        text: Option<&str>,
    ) -> Self {
        Self {
            uri: uri.clone(),
            version,
            language_id: language_id.map(|id| id.to_string()),
            text: match text {
                Some(text) => text.to_string(),
                None => String::new(),
            },
        }
    }
}

#[derive(Debug)]
pub struct TextDocumentCache {
    documents: Arc<RwLock<LruCache<String, TextDocumentCacheItem>>>,
}

impl Default for TextDocumentCache {
    fn default() -> Self {
        Self {
            documents: Arc::new(RwLock::new(LruCache::new(NonZero::new(1000).unwrap()))),
        }
    }
}

impl TextDocumentCache {
    pub fn get(&self, uri: &str) -> Option<TextDocumentCacheItem> {
        self.documents.write().unwrap().get(uri).cloned()
    }

    pub fn insert(&self, document: &TextDocumentItem) {
        let document = TextDocumentCacheItem::new(
            &document.uri,
            Some(document.version),
            Some(&document.language_id),
            Some(&document.text),
        );
        self.documents
            .write()
            .unwrap()
            .put(document.uri.to_string(), document);
    }

    pub fn update(&self, uri: &Url, text: &str) {
        let key = uri.to_string();
        let mut cache = self.documents.write().unwrap();
        let item = cache.get(&key);
        match item {
            Some(item) => {
                let new_item = TextDocumentCacheItem::new(
                    uri,
                    item.version,
                    item.language_id.as_deref(),
                    Some(text),
                );
                cache.put(key, new_item);
            }
            None => {
                let item = TextDocumentCacheItem::new(uri, None, None, Some(text));
                cache.put(key, item);
            }
        }
    }

    pub fn remove(&self, uri: &Url) {
        self.documents.write().unwrap().pop(uri.as_str());
    }

    pub fn cached_urls(&self) -> Vec<Url> {
        self.documents
            .read()
            .unwrap()
            .iter()
            .map(|(_, v)| v.uri.clone())
            .collect()
    }
}
