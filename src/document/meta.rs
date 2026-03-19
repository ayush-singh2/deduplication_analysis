//! Document metadata extraction
//! Extracts basic metadata from text documents

use std::fs::File;
use std::io::{Read, Result};
use sha1::Digest;

pub struct DocMeta {
    pub path: String,
    pub size: u64,
    pub sha1: String,
}

/// Extract metadata for a document file
pub fn extract_meta(path: &str) -> Result<DocMeta> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    let size = file.read_to_end(&mut buffer)? as u64;
    let sha1 = sha1::Sha1::digest(&buffer);
    Ok(DocMeta {
        path: path.to_string(),
        size,
        sha1: format!("{:x}", sha1),
    })
}
