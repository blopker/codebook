use std::num::NonZero;

use lru::LruCache;

#[derive(Default, Debug, Clone)]
pub struct TextDocumentCacheItem {
    pub lines: Vec<String>,
    pub uri: String,
    pub version: u32,
    pub language_id: String,
}

impl TextDocumentCacheItem {
    pub fn new(uri: &str, version: u32, language_id: &str, text: &str) -> Self {
        Self {
            uri: uri.to_string(),
            version,
            language_id: language_id.to_string(),
            lines: text.lines().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug)]
pub struct TextDocumentCache {
    documents: LruCache<String, TextDocumentCacheItem>,
}

impl TextDocumentCache {
    pub fn new() -> Self {
        Self {
            documents: LruCache::new(NonZero::new(1000).unwrap()),
        }
    }

    pub fn get(&mut self, uri: &str) -> Option<&TextDocumentCacheItem> {
        self.documents.get(uri)
    }

    pub fn insert(&mut self, uri: String, document: TextDocumentCacheItem) {
        self.documents.put(uri, document);
    }

    pub fn remove(&mut self, uri: &str) {
        self.documents.pop(uri);
    }
}
