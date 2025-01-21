use std::num::NonZero;

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
            language_id: match language_id {
                Some(id) => Some(id.to_string()),
                None => None,
            },
            text: match text {
                Some(text) => text.to_string(),
                None => String::new(),
            },
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

    pub fn insert(&mut self, document: &TextDocumentItem) {
        let document = TextDocumentCacheItem::new(
            &document.uri,
            Some(document.version),
            Some(&document.language_id),
            Some(&document.text),
        );
        self.documents.put(document.uri.to_string(), document);
    }

    pub fn update(&mut self, uri: &Url, text: &str) {
        let key = uri.to_string();
        let item = self.documents.get(&key);
        match item {
            Some(item) => {
                let new_item = TextDocumentCacheItem::new(
                    uri,
                    item.version,
                    item.language_id.as_deref(),
                    Some(text),
                );
                self.documents.put(key, new_item);
            }
            None => {
                let item = TextDocumentCacheItem::new(uri, None, None, Some(text));
                self.documents.put(key, item);
            }
        }
    }

    pub fn remove(&mut self, uri: &Url) {
        self.documents.pop(uri.as_str());
    }
}
