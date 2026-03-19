//! Document grouping
//! Groups documents by fingerprint similarity

use crate::document::fingerprint::{sha1_hash, ngram_hash};
use std::collections::HashMap;
use std::io::Result;

/// Group documents by SHA-1 hash
pub fn group_by_sha1(paths: &[&str]) -> Result<HashMap<String, Vec<String>>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for path in paths {
        let hash = sha1_hash(path)?;
        groups.entry(hash).or_default().push(path.to_string());
    }
    Ok(groups)
}

/// Group documents by n-gram hash
pub fn group_by_ngram(paths: &[&str], n: usize) -> Result<HashMap<String, Vec<String>>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for path in paths {
        let hash = ngram_hash(path, n)?;
        groups.entry(hash).or_default().push(path.to_string());
    }
    Ok(groups)
}
