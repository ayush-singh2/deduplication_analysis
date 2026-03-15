// Video deduplication module for Rust
// Place in dedupl-rs/video/mod.rs

pub mod meta;
pub mod fingerprint;
pub mod grouping;
pub mod exts;

use std::collections::HashSet;
use std::path::Path;
use crate::common::fs::walk_files_by_extension;
use self::exts::VIDEO_EXTS;

/// Find all video files under `root`.
pub fn find_video_files(root: &Path) -> Vec<std::path::PathBuf> {
	let exts: HashSet<&str> = VIDEO_EXTS.iter().copied().collect();
	walk_files_by_extension(root, &exts)
}
