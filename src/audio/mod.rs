//! Audio deduplication module — fingerprinting via `fpcalc`, metadata via
//! `ffprobe`, and quality-based duplicate selection.

pub mod fingerprint;
pub mod grouping;
pub mod meta;
pub mod quality;

pub use fingerprint::{generate_fingerprint, probe_ffprobe};
pub use grouping::group_duplicates;
pub use meta::{AudioMeta, FingerprintEntry, AUDIO_EXTS, LOSSLESS_CODECS};
pub use quality::select_best_quality;

use std::collections::HashSet;
use std::path::Path;

use crate::common::{walk_files_by_extension, Hasher};

/// Find all audio files under `root`.
pub fn find_audio_files(root: &Path) -> Vec<std::path::PathBuf> {
    let exts: HashSet<&str> = AUDIO_EXTS.iter().copied().collect();
    walk_files_by_extension(root, &exts)
}

/// Concrete hasher for audio files.
#[derive(Debug, Default)]
pub struct AudioHasher;

impl Hasher for AudioHasher {
    type Meta = FingerprintEntry;

    fn scan(&self, path: &Path) -> anyhow::Result<Option<Self::Meta>> {
        Ok(fingerprint::scan_audio_file(path))
    }

    fn extensions(&self) -> &'static [&'static str] {
        AUDIO_EXTS
    }
}
