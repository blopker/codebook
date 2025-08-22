use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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
    pub fn has_changed(&self) -> bool {
        let Some(path) = &self.path else {
            return false;
        };

        match fs::metadata(path) {
            Ok(metadata) => {
                let modified = metadata.modified().ok();
                let size = metadata.len();

                // If we have no previous state, consider it changed
                if self.last_modified.is_none() && self.last_size.is_none() {
                    return true;
                }

                modified != self.last_modified || Some(size) != self.last_size
            }
            Err(_) => {
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

        if self.content.is_none() || self.has_changed() {
            let content = loader(path)?;
            Ok(self.with_content(content))
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
        if !self.has_changed() {
            return Ok((self, false));
        }

        let path = self
            .path
            .as_ref()
            .ok_or_else(|| "No path configured for watched file".to_string())?;

        // Try to load the file
        match loader(path) {
            Ok(content) => Ok((self.with_content(content), true)),
            Err(_) => {
                // File might be deleted or unreadable, clear the content
                Ok((self.cleared(), true))
            }
        }
    }

    /// Get the current content
    pub fn content(&self) -> Option<&T> {
        self.content.as_ref()
    }

    /// Replace the content without reloading from file
    pub fn with_content_value(self, content: T) -> Self {
        self.with_content(content)
    }

    /// Private: Create a new instance with updated content and file metadata
    fn with_content(self, content: T) -> Self {
        let (last_modified, last_size) = if let Some(path) = &self.path {
            if let Ok(metadata) = fs::metadata(path) {
                (metadata.modified().ok(), Some(metadata.len()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
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
