//! Common module — shared configuration, security, filesystem, and command
//! utilities used by the audio and image sub-modules.

pub mod command;
pub mod config;
pub mod fs;
pub mod security;
pub mod stats;

use std::path::Path;

// Re-export the most-used items at the `common::` level.
pub use command::execute_command;
pub use config::DeduplicationConfig;
pub use fs::{file_sha1_hash, walk_files_by_extension};
pub use security::validate_path_security;
pub use stats::{format_file_size, DuplicateStats};

/// Generic content-fingerprinting trait implemented by each media module.
///
/// Both `AudioHasher` and `ImageHasher` implement this trait, giving the CLI
/// and test harness a uniform interface for scanning files.
pub trait Hasher: Send + Sync {
    /// The metadata type returned by a successful scan.
    type Meta: Send + Sync;

    /// Scan a single file and return its metadata/fingerprint.
    ///
    /// Returns `Ok(None)` when the file cannot be processed (corrupt, wrong
    /// format, etc.) rather than an `Err` — only true unexpected failures
    /// should propagate errors.
    fn scan(&self, path: &Path) -> anyhow::Result<Option<Self::Meta>>;

    /// The set of lowercase file extensions (including the leading dot)
    /// that this hasher knows how to process.
    fn extensions(&self) -> &'static [&'static str];
}
