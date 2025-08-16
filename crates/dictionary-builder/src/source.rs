//! Source file retrieval.

use std::fs;
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use tracing::{debug, info};
use url::Url;

use crate::Result;
use crate::error::Error;

/// Fetcher for source files
pub struct SourceFetcher {
    cache_dir: Option<PathBuf>,
    client: Client,
}

impl SourceFetcher {
    /// Create a new source fetcher
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        // Create cache directory if it doesn't exist
        if let Some(ref dir) = cache_dir
            && !dir.exists()
        {
            fs::create_dir_all(dir).expect("Failed to create cache directory");
        }

        Self {
            cache_dir,
            client: Client::new(),
        }
    }

    /// Fetch a file from a source
    pub fn fetch_file(&self, repository: &str, path: &str, output: &Path) -> Result<()> {
        // Check cache first if enabled
        if let Some(cached_path) = self.get_cached_path(repository, path)
            && cached_path.exists()
        {
            debug!("Using cached file: {}", cached_path.display());
            fs::copy(&cached_path, output)?;
            return Ok(());
        }

        // Not in cache, need to download
        let url = self.construct_url(repository, path)?;
        debug!("Fetching {} from {}", path, url);

        let response = self.client.get(url).send()?;
        if !response.status().is_success() {
            return Err(Error::Fetch(format!(
                "Failed to fetch {}: {}",
                path,
                response.status()
            )));
        }

        let content = response.bytes()?;
        fs::write(output, content)?;

        // Cache the file if caching is enabled
        if let Some(cache_path) = self.get_cached_path(repository, path) {
            if let Some(parent) = cache_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(output, &cache_path)?;
            debug!("Cached file at {}", cache_path.display());
        }

        info!("Fetched {} to {}", path, output.display());
        Ok(())
    }

    /// Clean the cache directory
    pub fn clean_cache(&self) -> Result<()> {
        if let Some(ref dir) = self.cache_dir
            && dir.exists()
        {
            fs::remove_dir_all(dir)?;
            fs::create_dir_all(dir)?;
            info!("Cleaned cache directory: {}", dir.display());
        }
        Ok(())
    }

    // Helper to get the path in the cache
    fn get_cached_path(&self, repository: &str, path: &str) -> Option<PathBuf> {
        self.cache_dir.as_ref().map(|dir| {
            // Create a path that includes the repository and file path
            // We need to sanitize the repository URL to use as a directory name
            let repo_dir = repository
                .replace("https://", "")
                .replace("http://", "")
                .replace('/', "-");

            dir.join(repo_dir).join(path)
        })
    }

    // Helper to construct a URL for fetching from GitHub
    fn construct_url(&self, repository: &str, path: &str) -> Result<String> {
        // Handle GitHub repositories
        if repository.contains("github.com") {
            // Convert from:
            // https://github.com/user/repo
            // To:
            // https://raw.githubusercontent.com/user/repo/main/path

            let url = Url::parse(repository)
                .map_err(|e| Error::Fetch(format!("Invalid repository URL: {e}")))?;

            let segments: Vec<_> = url
                .path_segments()
                .ok_or_else(|| Error::Fetch("Invalid repository URL path".to_string()))?
                .collect();

            if segments.len() < 2 {
                return Err(Error::Fetch(format!(
                    "Invalid GitHub repository URL: {repository}"
                )));
            }

            let user = segments[0];
            let repo = segments[1];

            // Default to 'main' branch
            let branch = "main";

            return Ok(format!(
                "https://raw.githubusercontent.com/{user}/{repo}/{branch}/{path}"
            ));
        }

        // For other URLs, just join the path
        Ok(format!("{}/{path}", repository.trim_end_matches('/')))
    }
}
