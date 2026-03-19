//! Document fingerprinting
//! Generates fingerprints for text documents using content hashing and similarity metrics

use std::fs::File;
use std::io::{Read, Result};
use sha1::{Sha1, Digest};

/// Compute SHA-1 hash for a document file
pub fn sha1_hash(path: &str) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mut hasher = Sha1::new();
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute simple content fingerprint (e.g., n-gram hash)
pub fn ngram_hash(path: &str, n: usize) -> Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let ngrams: Vec<&str> = buffer.as_bytes()
        .windows(n)
        .map(|w| std::str::from_utf8(w).unwrap_or(""))
        .collect();
    let mut hasher = Sha1::new();
    for ng in ngrams {
        hasher.update(ng.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}
