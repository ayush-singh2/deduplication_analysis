//! **dedupl** — High-performance media file deduplication using perceptual
//! fingerprinting.
//!
//! A Rust toolkit for detecting and removing duplicate audio, image, and video
//! files using perceptual and content-based fingerprinting.

pub mod audio;
pub mod common;
pub mod error;
pub mod image;
pub mod video;
pub mod document;

// Re-export the most commonly used items at crate root.
pub use common::{DeduplicationConfig, DuplicateStats, Hasher};
pub use error::DeduplError;
