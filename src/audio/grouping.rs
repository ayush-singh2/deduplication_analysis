//! Audio duplicate grouping — mirrors Python's `group_duplicates`.

use std::collections::HashMap;

use super::meta::{AudioMeta, FingerprintEntry};

/// Group audio files by fingerprint hash and duration bucket.
///
/// `duration_tolerance` controls how aggressively durations are bucketed.
/// Each entry is assigned `key = "{fp_hash}:{duration / tolerance}"`.
pub fn group_duplicates(
    entries: &[FingerprintEntry],
    duration_tolerance: i64,
) -> HashMap<String, Vec<AudioMeta>> {
    let mut groups: HashMap<String, Vec<AudioMeta>> = HashMap::new();

    let divisor = duration_tolerance.max(1);

    for entry in entries {
        let duration_bucket = entry.duration / divisor;
        let key = format!("{}:{}", entry.fp_hash, duration_bucket);
        groups
            .entry(key)
            .or_default()
            .push(entry.meta.clone());
    }

    groups
}
