//! Filesystem utilities — file hashing and directory walking.

use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha1::Sha1;
use digest::Digest;
use tracing::{debug, error};
use walkdir::WalkDir;

use super::security::validate_path_security;

/// Calculate the SHA-1 hash of a file (1 MiB chunks).
///
/// Returns `None` on I/O error rather than propagating — mirrors the Python
/// `file_sha1_hash` function.
pub fn file_sha1_hash(path: &Path) -> Option<String> {
    file_sha1_hash_with_chunk(path, 1024 * 1024)
}

/// Calculate SHA-1 with a configurable chunk size (useful for testing).
pub fn file_sha1_hash_with_chunk(path: &Path, chunk_size: usize) -> Option<String> {
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Error hashing file {}: {e}", path.display());
            return None;
        }
    };

    let mut hasher = Sha1::new();
    let mut buf = vec![0u8; chunk_size];

    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(e) => {
                error!("Error reading file {}: {e}", path.display());
                return None;
            }
        }
    }

    Some(format!("{:x}", hasher.finalize()))
}

/// Recursively walk `root` and collect files whose lowercase extension is in
/// `extensions`.  Paths that fail the security check are silently skipped.
pub fn walk_files_by_extension(root: &Path, extensions: &HashSet<&str>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = match path.extension() {
            Some(e) => format!(".{}", e.to_string_lossy().to_lowercase()),
            None => continue,
        };

        if extensions.contains(ext.as_str()) && validate_path_security(path) {
            files.push(path.to_path_buf());
        }
    }

    debug!("Found {} matching files in {}", files.len(), root.display());
    files
}
