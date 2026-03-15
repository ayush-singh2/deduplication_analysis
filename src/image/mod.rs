//! Image deduplication module — perceptual hashing, exact SHA-1 grouping,
//! and quality-based duplicate selection.

pub mod grouping;
pub mod meta;
pub mod phash;
pub mod quality;

pub use grouping::{group_by_exact_hash, group_by_perceptual_hash};
pub use meta::{ImageMeta, IMAGE_EXTS};
pub use phash::{compute_perceptual_hash, read_image_dimensions};
pub use quality::select_best_quality;

use std::collections::HashSet;
use std::path::Path;

use crate::common::{walk_files_by_extension, Hasher};

/// Find all image files under `root`.
pub fn find_image_files(root: &Path) -> Vec<std::path::PathBuf> {
    let exts: HashSet<&str> = IMAGE_EXTS.iter().copied().collect();
    walk_files_by_extension(root, &exts)
}

/// Concrete hasher for image files.
#[derive(Debug, Default)]
pub struct ImageHasher {
    pub hash_size: u32,
}

impl ImageHasher {
    pub fn new(hash_size: u32) -> Self {
        Self { hash_size }
    }
}

impl Hasher for ImageHasher {
    type Meta = ImageMeta;

    fn scan(&self, path: &Path) -> anyhow::Result<Option<Self::Meta>> {
        Ok(phash::scan_image_file(path, self.hash_size))
    }

    fn extensions(&self) -> &'static [&'static str] {
        IMAGE_EXTS
    }
}
