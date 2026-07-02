use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Snapshot of a file's change-detection metadata; None if the file is inaccessible.
fn disk_meta(path: &Path) -> Option<(Option<SystemTime>, u64)> {
    fs::metadata(path)
        .ok()
        .map(|m| (m.modified().ok(), m.len()))
}

/// Simple immutable file watcher that tracks changes and loads content on demand
#[derive(Debug, Clone)]
pub struct WatchedFile<T: Clone> {
    path: Option<PathBuf>,
    content: Option<T>,
    last_modified: Option<SystemTime>,
    last_size: Option<u64>,
}

impl<T: Clone> WatchedFile<T> {
    /// Create a new watched file with the given path
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            content: None,
            last_modified: None,
            last_size: None,
        }
    }

    /// Get the path of the watched file
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Check if the file has changed since last check
    #[cfg(test)]
    pub fn has_changed(&self) -> bool {
        let Some(path) = &self.path else {
            return false;
        };
        self.meta_differs(&disk_meta(path))
    }

    /// Compare previously recorded metadata against a fresh disk snapshot.
    fn meta_differs(&self, meta: &Option<(Option<SystemTime>, u64)>) -> bool {
        match meta {
            Some((modified, size)) => {
                // If we have no previous state, consider it changed
                if self.last_modified.is_none() && self.last_size.is_none() {
                    return true;
                }
                *modified != self.last_modified || Some(*size) != self.last_size
            }
            None => {
                // File doesn't exist or is inaccessible
                // Consider it changed if we previously had content
                self.last_modified.is_some() || self.last_size.is_some()
            }
        }
    }

    /// Load the file content, returning a new instance with updated content
    pub fn load<F>(self, loader: F) -> Result<Self, String>
    where
        F: FnOnce(&Path) -> Result<T, String>,
    {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| "No path configured for watched file".to_string())?;

        // Stat before reading: if a write races the read, the stamp stays older
        // than the file and the next check reloads, instead of missing the write.
        let meta = disk_meta(path);
        if self.content.is_none() || self.meta_differs(&meta) {
            let content = loader(path)?;
            Ok(self.with_content_meta(content, meta))
        } else {
            Ok(self)
        }
    }

    /// Load the file content if it has changed
    /// Returns (new_instance, was_changed)
    pub fn reload_if_changed<F>(self, loader: F) -> Result<(Self, bool), String>
    where
        F: FnOnce(&Path) -> Result<T, String>,
    {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| "No path configured for watched file".to_string())?;

        // Stat before reading (see load() for why)
        let meta = disk_meta(path);
        if !self.meta_differs(&meta) {
            return Ok((self, false));
        }

        match loader(path) {
            Ok(content) => Ok((self.with_content_meta(content, meta), true)),
            Err(_) if meta.is_none() => {
                // File was deleted, clear the content
                Ok((self.cleared(), true))
            }
            Err(e) => {
                // File exists but is unreadable or invalid (e.g. mid-edit TOML).
                // Keep the last good content; the stale stamp means we retry on
                // the next reload.
                log::warn!("Keeping previous config for {}: {e}", path.display());
                Ok((self, false))
            }
        }
    }

    /// Get the current content
    pub fn content(&self) -> Option<&T> {
        self.content.as_ref()
    }

    /// Replace the content without reloading from file
    pub fn with_content_value(self, content: T) -> Self {
        let meta = self.path.as_deref().and_then(disk_meta);
        self.with_content_meta(content, meta)
    }

    /// Refresh the change-detection stamp from disk, keeping the current content.
    /// Call after writing the file so the write isn't detected as an external change.
    pub fn restamped(self) -> Self {
        let (last_modified, last_size) = match self.path.as_deref().and_then(disk_meta) {
            Some((modified, size)) => (modified, Some(size)),
            None => (None, None),
        };
        Self {
            last_modified,
            last_size,
            ..self
        }
    }

    /// Private: Create a new instance with updated content and the given file metadata
    fn with_content_meta(self, content: T, meta: Option<(Option<SystemTime>, u64)>) -> Self {
        let (last_modified, last_size) = match meta {
            Some((modified, size)) => (modified, Some(size)),
            None => (None, None),
        };

        Self {
            path: self.path,
            content: Some(content),
            last_modified,
            last_size,
        }
    }

    /// Private: Create a new instance with cleared content
    fn cleared(self) -> Self {
        Self {
            path: self.path,
            content: None,
            last_modified: None,
            last_size: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_watched_file_basic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create initial file
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "initial content").unwrap();
        drop(file);

        let watched = WatchedFile::<String>::new(Some(file_path.clone()));

        // First load
        let watched = watched
            .load(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert_eq!(watched.content().map(|s| s.trim()), Some("initial content"));

        // Check no change
        assert!(!watched.has_changed());

        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "modified content").unwrap();
        drop(file);

        // Should detect change
        assert!(watched.has_changed());

        // Reload
        let watched = watched
            .load(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert_eq!(
            watched.content().map(|s| s.trim()),
            Some("modified content")
        );
    }

    #[test]
    fn test_watched_file_deleted() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create and load file
        fs::write(&file_path, "content").unwrap();
        let watched = WatchedFile::<String>::new(Some(file_path.clone()));
        let watched = watched
            .load(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert_eq!(watched.content().map(|s| s.trim()), Some("content"));

        // Delete file
        fs::remove_file(&file_path).unwrap();

        // Should detect change (deletion)
        assert!(watched.has_changed());

        // Reload should clear the content
        let (watched, changed) = watched
            .reload_if_changed(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert!(changed);

        // Content should now be None
        assert!(watched.content().is_none());
    }

    #[test]
    fn test_watched_file_no_path() {
        let watched = WatchedFile::<String>::new(None);
        assert!(!watched.has_changed());
        assert!(watched.content().is_none());
    }
}
