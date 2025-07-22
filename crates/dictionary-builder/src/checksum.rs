//! Checksum calculation utilities.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};
use tracing::debug;

use crate::Result;

/// Calculate SHA-256 checksum of a file
pub fn calculate_sha256(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    let hash_str = format!("sha256:{:x}", hash);

    debug!("Calculated checksum for {}: {hash_str}", path.display());
    Ok(hash_str)
}
