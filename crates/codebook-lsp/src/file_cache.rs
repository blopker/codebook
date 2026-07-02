use std::{collections::HashMap, sync::RwLock};

use log::debug;
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

/// Cache of the client's open documents, mirroring the didOpen/didClose
/// lifecycle: didOpen is the only way in, didClose the only way out. That
/// makes the size exactly the client's open-document count — bounded by the
/// protocol, with no eviction that could orphan a still-open document.
#[derive(Debug, Default)]
pub struct TextDocumentCache {
    documents: RwLock<HashMap<String, TextDocumentCacheItem>>,
}

impl TextDocumentCache {
    pub fn get(&self, uri: &str) -> Option<TextDocumentCacheItem> {
        self.documents.read().unwrap().get(uri).cloned()
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
            .insert(document.uri.to_string(), document);
    }

    /// Update an open document's text. `version` replaces the stored version
    /// when Some (didChange); None keeps the existing one (didSave, which
    /// carries no version). Updates for documents the client never opened are
    /// dropped — inserting here would create entries no didClose removes.
    pub fn update(&self, uri: &Url, text: &str, version: Option<i32>) {
        let key = uri.to_string();
        let mut cache = self.documents.write().unwrap();
        match cache.get_mut(&key) {
            Some(item) => {
                item.text = text.to_string();
                if version.is_some() {
                    item.version = version;
                }
            }
            None => {
                debug!("Ignoring update for document that was never opened: {uri}");
            }
        }
    }

    pub fn remove(&self, uri: &Url) {
        self.documents.write().unwrap().remove(uri.as_str());
    }

    pub fn cached_urls(&self) -> Vec<Url> {
        self.documents
            .read()
            .unwrap()
            .values()
            .map(|v| v.uri.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(uri: &Url) -> TextDocumentItem {
        TextDocumentItem {
            uri: uri.clone(),
            language_id: "rust".to_string(),
            version: 1,
            text: "hello".to_string(),
        }
    }

    #[test]
    fn test_lifecycle_bounds_cache() {
        let cache = TextDocumentCache::default();
        let uri = Url::parse("file:///a.rs").unwrap();

        cache.insert(&doc(&uri));
        assert_eq!(cache.cached_urls().len(), 1);

        cache.remove(&uri);
        assert!(cache.cached_urls().is_empty());
        assert!(cache.get(uri.as_str()).is_none());
    }

    #[test]
    fn test_update_changes_text_and_version() {
        let cache = TextDocumentCache::default();
        let uri = Url::parse("file:///a.rs").unwrap();
        cache.insert(&doc(&uri));

        cache.update(&uri, "changed", Some(2));
        let item = cache.get(uri.as_str()).unwrap();
        assert_eq!(item.text, "changed");
        assert_eq!(item.version, Some(2));

        // didSave carries no version; the stored one must survive
        cache.update(&uri, "saved", None);
        let item = cache.get(uri.as_str()).unwrap();
        assert_eq!(item.text, "saved");
        assert_eq!(item.version, Some(2));
    }

    #[test]
    fn test_update_for_unopened_document_is_dropped() {
        let cache = TextDocumentCache::default();
        let uri = Url::parse("file:///never-opened.rs").unwrap();

        cache.update(&uri, "text", Some(1));

        // No entry may be created outside didOpen — nothing would remove it
        assert!(cache.get(uri.as_str()).is_none());
        assert!(cache.cached_urls().is_empty());
    }
}
