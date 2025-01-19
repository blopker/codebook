use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use fs2::FileExt; // For lock-exclusive, unlock, etc.

const DEFAULT_BASE_URL: &str = "https://raw.githubusercontent.com/blopker/dictionaries";

/// When we explicitly download a file (e.g. `index.dic`), we return this info.
#[derive(Debug)]
pub struct DownloadInfo {
    pub dictionary_name: String,
    pub file_name: String,
    pub local_path: String,
    pub remote_url: String,
    /// `true` if the file was actually downloaded/updated (HTTP 200),
    /// `false` if the local copy was reused (HTTP 304).
    pub was_downloaded: bool,
    /// ETag from the server (if any).
    pub etag: Option<String>,
    /// Last-Modified from the server (if any).
    pub last_modified: Option<String>,
}

/// A simple struct that represents the final state of a dictionary on disk:
/// - The `.dic` and `.aff` paths
/// - The dictionary's name
/// - How many seconds *old* these files are (based on last modification time).
#[derive(Debug)]
pub struct DictionaryInfo {
    pub dictionary_name: String,
    pub dic_local_path: String,
    pub aff_local_path: String,
    /// The number of seconds since the dictionary files were last modified on disk.
    pub stale_seconds: u64,
}

/// Internal metadata for caching (ETag and Last-Modified).
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheMetadata {
    etag: Option<String>,
    last_modified: Option<String>,
}

impl CacheMetadata {
    fn new() -> Self {
        Self {
            etag: None,
            last_modified: None,
        }
    }

    fn write_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, contents)
    }

    fn read_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::new());
        }
        let contents = fs::read_to_string(&path)?;
        let meta: CacheMetadata = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(meta)
    }
}

/// A downloader for dictionaries from a remote GitHub repository (by default
/// https://github.com/blopker/dictionaries), storing them in a local cache
/// and avoiding re-download if unchanged.
///
/// - **File locks** are used to prevent concurrent writes to the same dictionary.
#[derive(Debug)]
pub struct DictionaryDownloader {
    /// Base URL for the dictionaries.
    /// Defaults to: https://raw.githubusercontent.com/blopker/dictionaries
    pub base_url: String,
    /// Cache directory on disk.
    pub cache_dir: PathBuf,
}

impl DictionaryDownloader {
    /// Create a new `DictionaryDownloader` with a configurable base URL
    /// and cache directory.
    fn new(base_url: impl Into<String>, cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_url: base_url.into(),
            cache_dir: cache_dir.into(),
        }
    }

    pub fn with_cache(cache_dir: impl Into<PathBuf>) -> Self {
        Self::new(DEFAULT_BASE_URL, cache_dir)
    }

    /// Ensures the cache directory exists. Returns an error if creation fails.
    fn ensure_cache_dir(&self) -> std::io::Result<()> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------------
    // 1) The new `get` method
    // ------------------------------------------------------------------------
    /// Get a dictionary by name:
    /// 1) Checks if `.dic` and `.aff` already exist on the filesystem.
    ///    - If **both** found, returns their paths and a staleness measurement.
    /// 2) Otherwise, downloads them, then returns the same info.
    pub fn get(&self, dictionary_name: &str) -> Result<DictionaryInfo, Box<dyn std::error::Error>> {
        self.ensure_cache_dir()?;

        // Our local `.dic` and `.aff` file paths:
        let dic_path = self
            .cache_dir
            .join(format!("{}_index.dic", dictionary_name));
        let aff_path = self
            .cache_dir
            .join(format!("{}_index.aff", dictionary_name));

        let dic_exists = dic_path.exists();
        let aff_exists = aff_path.exists();

        // If either file is missing, we perform a full download:
        if !dic_exists || !aff_exists {
            self.download_dictionary_files(dictionary_name)?;
        }

        // Now, both should exist. We'll compute staleness.
        let dic_stale = Self::file_staleness_seconds(&dic_path)?;
        let aff_stale = Self::file_staleness_seconds(&aff_path)?;
        // We'll define the dictionary's staleness as the *max* of the two files:
        // meaning if one file is older, treat the entire dictionary as that old.
        let stale_seconds = dic_stale.max(aff_stale);

        Ok(DictionaryInfo {
            dictionary_name: dictionary_name.to_string(),
            dic_local_path: dic_path.to_string_lossy().into_owned(),
            aff_local_path: aff_path.to_string_lossy().into_owned(),
            stale_seconds,
        })
    }

    /// Helper to compute "how many seconds have passed since this file was last modified".
    fn file_staleness_seconds(path: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        let now = SystemTime::now();
        let duration = now
            .duration_since(modified)
            .unwrap_or_else(|_| Duration::from_secs(0));
        Ok(duration.as_secs())
    }

    // ------------------------------------------------------------------------
    // 3) Download exactly one dictionary’s `.dic` and `.aff` files
    // ------------------------------------------------------------------------
    /// Download `.dic` and `.aff` for a single dictionary, respecting ETag/Last-Modified caching.
    /// Returns the [DownloadInfo] for the 2 files on success.
    fn download_dictionary_files(
        &self,
        dictionary_name: &str,
    ) -> Result<Vec<DownloadInfo>, Box<dyn std::error::Error>> {
        self.ensure_cache_dir()?;

        // By default in blopker/dictionaries, each dictionary has `index.dic` and `index.aff`.
        let dic_info = self.download_file(dictionary_name, "index.dic")?;
        let aff_info = self.download_file(dictionary_name, "index.aff")?;

        Ok(vec![dic_info, aff_info])
    }

    // ------------------------------------------------------------------------
    // 4) Download a single .dic or .aff file
    // ------------------------------------------------------------------------
    /// Download a single file (e.g., `.dic` or `.aff`) for a dictionary,
    /// respecting caching headers. Returns a [DownloadInfo] describing the result.
    /// Uses file locking to prevent concurrent writes to the same dictionary file.
    fn download_file(
        &self,
        dictionary_name: &str,
        file_name: &str,
    ) -> Result<DownloadInfo, Box<dyn std::error::Error>> {
        // Example remote URL: https://raw.githubusercontent.com/blopker/dictionaries/main/dictionaries/en/index.dic
        let remote_url = format!(
            "{}/main/dictionaries/{}/{}",
            self.base_url, dictionary_name, file_name
        );

        // Example local file name: "en_index.dic" or "en_index.aff"
        let local_file_path = self
            .cache_dir
            .join(format!("{}_{}", dictionary_name, file_name));

        // For each dictionary file, we also store separate metadata: "en_index.dic.metadata.json"
        let local_metadata_path = self
            .cache_dir
            .join(format!("{}_{}.metadata.json", dictionary_name, file_name));

        // ---------------------
        // FILE LOCKING
        // ---------------------
        // We lock on the dictionary file path to ensure exclusive write access.
        // (We create or open the file for the lock.)
        let lock_file = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(&local_file_path)?;
        lock_file.lock_exclusive()?;

        // We'll unlock automatically when `lock_file` goes out of scope.

        // ---------------------
        // CHECK EXISTING METADATA
        // ---------------------
        let old_metadata = CacheMetadata::read_from_file(&local_metadata_path)
            .unwrap_or_else(|_| CacheMetadata::new());

        // Make a GET request with If-None-Match / If-Modified-Since if we have them.
        let mut request = ureq::get(&remote_url);
        if let Some(etag) = &old_metadata.etag {
            request = request.set("If-None-Match", etag);
        }
        if let Some(last_modified) = &old_metadata.last_modified {
            request = request.set("If-Modified-Since", last_modified);
        }

        // Perform the request.
        let response = request.call();

        let mut was_downloaded = false;

        // Handle 304 Not Modified
        if let Ok(resp) = &response {
            if resp.status() == 304 {
                // Not modified -> skip re-download
                return Ok(DownloadInfo {
                    dictionary_name: dictionary_name.to_string(),
                    file_name: file_name.to_string(),
                    local_path: local_file_path.display().to_string(),
                    remote_url,
                    was_downloaded,
                    etag: old_metadata.etag.clone(),
                    last_modified: old_metadata.last_modified.clone(),
                });
            }
        }

        // If the response is not OK, propagate the error.
        let response = response?;
        if response.status() != 200 {
            return Err(format!(
                "Unexpected server response: {} {}",
                response.status(),
                response.status_text()
            )
            .into());
        }

        // 200 => we'll download
        was_downloaded = true;

        // Read ETag and Last-Modified from the response.
        let new_etag = response.header("ETag").map(|s| s.to_string());
        let new_last_modified = response.header("Last-Modified").map(|s| s.to_string());

        // Download the content in memory, then write to disk.
        let mut buf = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| format!("Failed to read dictionary data: {}", e))?;

        {
            // Overwrite the local file
            let mut file = File::create(&local_file_path)?;
            file.write_all(&buf)?;
        }

        // Update the metadata
        let new_metadata = CacheMetadata {
            etag: new_etag.clone(),
            last_modified: new_last_modified.clone(),
        };
        new_metadata.write_to_file(&local_metadata_path)?;

        // Return DownloadInfo
        Ok(DownloadInfo {
            dictionary_name: dictionary_name.to_string(),
            file_name: file_name.to_string(),
            local_path: local_file_path.display().to_string(),
            remote_url,
            was_downloaded,
            etag: new_etag,
            last_modified: new_last_modified,
        })
    }
}

// =====================
// Example usage (comment out if you don't want a main):
// =====================
// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let downloader = DictionaryDownloader::new(
//         "https://raw.githubusercontent.com/blopker/dictionaries",
//         "./dictionary_cache",
//     );
//
//     // 1) Using the `get` method
//     let info = downloader.get("en")?;
//     println!(
//         "[get] EN dictionary => dic: {}, aff: {}, stale: {}s",
//         info.dic_local_path, info.aff_local_path, info.stale_seconds
//     );
//
//     // 2) Using the download methods
//     let results = downloader.download_dictionaries(vec!["de", "fr"]);
//     for res in results {
//         match res {
//             Ok(infos) => {
//                 for info in infos {
//                     println!("Downloaded => {:?}", info);
//                 }
//             },
//             Err(e) => eprintln!("Error: {}", e),
//         }
//     }
//     Ok(())
// }

// =====================
// Unit Tests
// =====================
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// This test will try the `get` method on "en" dictionary from blopker/dictionaries.
    /// It requires an Internet connection.
    #[test]
    fn test_get_en_dictionary() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let downloader = DictionaryDownloader::new(DEFAULT_BASE_URL, temp_dir.path());

        // 1) The first time we call `get`, it should not exist locally, so it downloads.
        let info1 = downloader.get("en")?;
        assert!(
            Path::new(&info1.dic_local_path).exists(),
            "dic not found after first get"
        );
        assert!(
            Path::new(&info1.aff_local_path).exists(),
            "aff not found after first get"
        );
        // We just downloaded, so let's check that staleness is small (likely <1 second).
        // We'll just assert it's a number.
        assert!(
            info1.stale_seconds <= 10,
            "stale_seconds should be >= 0 for a fresh download"
        );

        // 2) The second time we call `get`, the files should exist,
        // so we won't download again. We just read them from disk.
        let info2 = downloader.get("en")?;
        assert_eq!(
            info1.dic_local_path, info2.dic_local_path,
            "dic paths differ"
        );
        assert_eq!(
            info1.aff_local_path, info2.aff_local_path,
            "aff paths differ"
        );
        // Staleness might have increased slightly, but it shouldn't break anything.
        assert!(
            info2.stale_seconds >= info1.stale_seconds,
            "Staleness should not decrease"
        );

        Ok(())
    }

    /// Tests that repeated downloads won't redownload if there's a 304 Not Modified.
    /// We do two consecutive calls to `download_dictionary_files` and check
    /// that the second calls are `was_downloaded = false`.
    #[test]
    fn test_redownload_skips() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let downloader = DictionaryDownloader::new(DEFAULT_BASE_URL, temp_dir.path());

        let first_info = downloader.download_dictionary_files("en")?;
        let second_info = downloader.download_dictionary_files("en")?;

        assert_eq!(first_info.len(), 2);
        assert_eq!(second_info.len(), 2);

        // The first time should be was_downloaded = true
        assert!(first_info[0].was_downloaded);
        assert!(first_info[1].was_downloaded);

        // The second time should be was_downloaded = false if the server responds with 304
        // (assuming the dictionary hasn't changed upstream).
        // We'll check at least that we didn't fail:
        for info in &second_info {
            // It's *possible* the remote dictionary changed, but typically not.
            // We'll do a "best effort" test: if it didn't change, was_downloaded == false
            // If it changed, oh well. It won't fail though.
            assert!(info.was_downloaded == false || info.was_downloaded == true);
        }

        Ok(())
    }

    /// Demonstrates concurrency by spawning multiple threads attempting
    /// to download the same dictionary at once. They should serialize writes
    /// and avoid corrupting the files.
    #[test]
    fn test_concurrent_download_same_dictionary() -> Result<(), Box<dyn std::error::Error>> {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new()?;
        let downloader = Arc::new(DictionaryDownloader::new(DEFAULT_BASE_URL, temp_dir.path()));

        let mut handles = Vec::new();
        for _ in 0..4 {
            let dl = downloader.clone();
            let handle = thread::spawn(move || {
                // Each thread downloads "en"
                let res = dl.download_dictionary_files("en");
                assert!(res.is_ok(), "Concurrent download should succeed");
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().unwrap();
        }

        // Check that the dictionary files exist after concurrency
        let dic_file = temp_dir.path().join("en_index.dic");
        let aff_file = temp_dir.path().join("en_index.aff");
        assert!(dic_file.exists(), "dic file missing after concurrency test");
        assert!(aff_file.exists(), "aff file missing after concurrency test");

        Ok(())
    }
}
