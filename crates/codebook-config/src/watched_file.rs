use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::SystemTime;

/// Generic file watcher that tracks changes and loads content on demand
#[derive(Debug)]
pub struct WatchedFile<T> {
    path: RwLock<Option<PathBuf>>,
    content: RwLock<Option<T>>,
    last_modified: RwLock<Option<SystemTime>>,
    last_size: RwLock<Option<u64>>,
}

impl<T> WatchedFile<T> {
    /// Create a new watched file with the given path
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path: RwLock::new(path),
            content: RwLock::new(None),
            last_modified: RwLock::new(None),
            last_size: RwLock::new(None),
        }
    }

    /// Get the path of the watched file
    pub fn path(&self) -> Option<PathBuf> {
        self.path.read().unwrap().clone()
    }

    /// Set or update the path
    pub fn set_path(&self, path: Option<PathBuf>) {
        if *self.path.read().unwrap() != path {
            *self.path.write().unwrap() = path;
            *self.content.write().unwrap() = None;
            *self.last_modified.write().unwrap() = None;
            *self.last_size.write().unwrap() = None;
        }
    }

    /// Check if the file has changed since last check
    pub fn has_changed(&self) -> bool {
        let path_ref = self.path.read().unwrap();
        let Some(path) = path_ref.as_ref() else {
            return false;
        };

        match fs::metadata(path) {
            Ok(metadata) => {
                let modified = metadata.modified().ok();
                let size = metadata.len();

                let last_modified = self.last_modified.read().unwrap();
                let last_size = self.last_size.read().unwrap();

                // If we have no previous state, consider it changed
                if last_modified.is_none() && last_size.is_none() {
                    return true;
                }

                modified != *last_modified || Some(size) != *last_size
            }
            Err(_) => {
                // File doesn't exist or is inaccessible
                // Consider it changed if we previously had content
                self.last_modified.read().unwrap().is_some()
                    || self.last_size.read().unwrap().is_some()
            }
        }
    }

    /// Load the file content using the provided loader function
    /// Returns a reference to the loaded content
    pub fn load<F>(&self, loader: F) -> Result<T, String>
    where
        F: FnOnce(&Path) -> Result<T, String>,
        T: Clone,
    {
        let path = self
            .path
            .read()
            .unwrap()
            .as_ref()
            .ok_or_else(|| "No path configured for watched file".to_string())?
            .clone();

        if self.content.read().unwrap().is_none() || self.has_changed() {
            *self.content.write().unwrap() = Some(loader(&path)?);
            self.update_state();
        }

        Ok(self.content.read().unwrap().as_ref().unwrap().clone())
    }

    /// Load the file content if it has changed
    /// Returns true if reloaded (including when cleared due to deletion), false if unchanged
    pub fn reload_if_changed<F>(&self, loader: F) -> Result<bool, String>
    where
        F: FnOnce(&Path) -> Result<T, String>,
        T: Clone,
    {
        if !self.has_changed() {
            return Ok(false);
        }

        let path = self
            .path
            .read()
            .unwrap()
            .as_ref()
            .ok_or_else(|| "No path configured for watched file".to_string())?
            .clone();

        // Try to load the file
        match loader(&path) {
            Ok(content) => {
                *self.content.write().unwrap() = Some(content);
                self.update_state();
                Ok(true)
            }
            Err(_) => {
                // File might be deleted or unreadable, clear the content
                *self.content.write().unwrap() = None;
                *self.last_modified.write().unwrap() = None;
                *self.last_size.write().unwrap() = None;
                Ok(true) // Content changed (cleared)
            }
        }
    }

    /// Get the current content without reloading
    pub fn content(&self) -> Option<T>
    where
        T: Clone,
    {
        self.content.read().unwrap().clone()
    }

    /// Replace the content without reloading from file
    pub fn set_content(&self, content: T) {
        *self.content.write().unwrap() = Some(content);
        self.update_state();
    }

    /// Update the internal state to match the current file metadata
    fn update_state(&self) {
        if let Some(path) = self.path.read().unwrap().as_ref() {
            if let Ok(metadata) = fs::metadata(path) {
                *self.last_modified.write().unwrap() = metadata.modified().ok();
                *self.last_size.write().unwrap() = Some(metadata.len());
            } else {
                // File doesn't exist
                *self.last_modified.write().unwrap() = None;
                *self.last_size.write().unwrap() = None;
            }
        }
    }
}

impl<T: Clone> Clone for WatchedFile<T> {
    fn clone(&self) -> Self {
        Self {
            path: RwLock::new(self.path.read().unwrap().clone()),
            content: RwLock::new(self.content.read().unwrap().clone()),
            last_modified: RwLock::new(*self.last_modified.read().unwrap()),
            last_size: RwLock::new(*self.last_size.read().unwrap()),
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
        let content = watched
            .load(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert_eq!(content.trim(), "initial content");

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
        let content = watched
            .load(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert_eq!(content.trim(), "modified content");
    }

    #[test]
    fn test_watched_file_deleted() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create and load file
        fs::write(&file_path, "content").unwrap();
        let watched = WatchedFile::<String>::new(Some(file_path.clone()));
        let content = watched
            .load(|path| fs::read_to_string(path).map_err(|e| e.to_string()))
            .unwrap();
        assert_eq!(content.trim(), "content");

        // Delete file
        fs::remove_file(&file_path).unwrap();

        // Should detect change (deletion)
        assert!(watched.has_changed());

        // Reload should clear the content
        let changed = watched
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
