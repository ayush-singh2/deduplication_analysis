//! Image metadata struct — mirrors Python's `ImageMeta`.

use std::path::PathBuf;

/// Supported image file extensions (lowercase, with leading dot).
pub const IMAGE_EXTS: &[&str] = &[
    ".jpg", ".jpeg", ".png", ".webp", ".bmp", ".gif", ".tif", ".tiff", ".heic", ".heif",
];

/// Metadata for a single image file.
#[derive(Debug, Clone)]
pub struct ImageMeta {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub mtime: f64,
    pub sha1: String,
    /// Raw perceptual hash bits stored as a byte vector.
    /// `None` if the hash could not be computed.
    pub phash: Option<Vec<u8>>,
}

impl PartialEq for ImageMeta {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.width == other.width
            && self.height == other.height
            && self.size == other.size
            && self.sha1 == other.sha1
            && self.phash == other.phash
    }
}

impl Eq for ImageMeta {}

impl ImageMeta {
    /// Resolution (total pixel count).
    pub fn resolution(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Aspect ratio (width / height), or 0.0 if height is zero.
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f64 / self.height as f64
        }
    }
}
