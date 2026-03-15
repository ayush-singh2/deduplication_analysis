//! Image duplicate grouping — exact hash and perceptual hash.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::meta::ImageMeta;
use super::phash::hamming_distance;

/// Group images by exact SHA-1 hash.
///
/// Only groups with more than one member (i.e. actual duplicates) are returned.
pub fn group_by_exact_hash(items: &[ImageMeta]) -> Vec<Vec<ImageMeta>> {
    let mut groups: HashMap<&str, Vec<&ImageMeta>> = HashMap::new();

    for item in items {
        groups.entry(item.sha1.as_str()).or_default().push(item);
    }

    groups
        .into_values()
        .filter(|g| g.len() > 1)
        .map(|g| g.into_iter().cloned().collect())
        .collect()
}

/// Group images by perceptual hash similarity using Hamming distance.
///
/// - `threshold`: maximum Hamming distance to consider "similar".
/// - `check_aspect_ratio`: when `true`, reject pairs whose aspect ratios
///   differ by more than ~10%.
pub fn group_by_perceptual_hash(
    items: &[ImageMeta],
    threshold: u32,
    check_aspect_ratio: bool,
) -> Vec<Vec<ImageMeta>> {
    // Filter to items with a valid perceptual hash.
    let mut with_phash: Vec<&ImageMeta> = items.iter().filter(|m| m.phash.is_some()).collect();

    // Sort by quality (highest resolution first) for consistent grouping.
    with_phash.sort_by(|a, b| {
        b.resolution()
            .cmp(&a.resolution())
            .then_with(|| b.size.cmp(&a.size))
            .then_with(|| {
                b.mtime
                    .partial_cmp(&a.mtime)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut groups: Vec<Vec<ImageMeta>> = Vec::new();

    for (i, base) in with_phash.iter().enumerate() {
        if visited.contains(&base.path) {
            continue;
        }

        let mut group = vec![(*base).clone()];
        visited.insert(base.path.clone());

        let base_hash = base.phash.as_ref().expect("filtered above");

        for candidate in &with_phash[i + 1..] {
            if visited.contains(&candidate.path) {
                continue;
            }

            let cand_hash = candidate.phash.as_ref().expect("filtered above");
            let distance = hamming_distance(base_hash, cand_hash);

            if distance <= threshold {
                // Optional aspect-ratio check.
                if check_aspect_ratio {
                    let ar_base = base.aspect_ratio();
                    let ar_cand = candidate.aspect_ratio();

                    if ar_base > 0.0 && ar_cand > 0.0 {
                        let ratio = ar_base / ar_cand;
                        if !(0.9..=1.11).contains(&ratio) {
                            continue;
                        }
                    }
                }

                group.push((*candidate).clone());
                visited.insert(candidate.path.clone());
            }
        }

        if group.len() > 1 {
            groups.push(group);
        }
    }

    groups
}
