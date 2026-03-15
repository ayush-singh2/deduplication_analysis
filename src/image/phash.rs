//! Perceptual hashing and dimension reading for images.

use std::path::Path;

use image::GenericImageView;
use img_hash::{HashAlg, HasherConfig};
use tracing::debug;

use crate::common::fs::file_sha1_hash;
use crate::common::security::validate_path_security;

use super::meta::ImageMeta;

/// Compute a perceptual hash (pHash) of an image.
///
/// Returns the hash as a raw byte vector, or `None` on error.
/// `hash_size` controls the granularity (default 8 in `img_hash`).
///
/// We decode with `image` 0.25 (which has broad format support), then
/// convert the raw pixel buffer into the `img_hash::image` (0.23) type
/// so the hasher can consume it.
pub fn compute_perceptual_hash(path: &Path, hash_size: u32) -> Option<Vec<u8>> {
    if !validate_path_security(path) {
        return None;
    }

    // Decode using image 0.25 (supports JPEG, PNG, BMP, WebP, GIF, TIFF…).
    let img = match image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
    {
        Ok(i) => i,
        Err(e) => {
            debug!("Failed to decode image {}: {e}", path.display());
            return None;
        }
    };

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let raw = rgba.into_raw();

    // Construct an img_hash::image (0.23) RgbaImage from the raw pixel data.
    let old_img = img_hash::image::RgbaImage::from_raw(w, h, raw)?;

    let hasher = HasherConfig::new()
        .hash_size(hash_size, hash_size)
        .hash_alg(HashAlg::Gradient)
        .to_hasher();

    let hash = hasher.hash_image(&old_img);
    Some(hash.as_bytes().to_vec())
}

/// Read image dimensions (width, height).
///
/// Returns `(0, 0)` on any error — mirrors the Python behaviour.
pub fn read_image_dimensions(path: &Path) -> (u32, u32) {
    let img = match image::ImageReader::open(path)
        .ok()
        .and_then(|r| r.with_guessed_format().ok())
        .and_then(|r| r.decode().ok())
    {
        Some(i) => i,
        None => {
            debug!("Failed to read dimensions for {}", path.display());
            return (0, 0);
        }
    };

    img.dimensions()
}

/// Scan an image file: compute SHA-1, dimensions, and perceptual hash.
pub fn scan_image_file(path: &Path, hash_size: u32) -> Option<ImageMeta> {
    let sha1 = file_sha1_hash(path)?;
    let (width, height) = read_image_dimensions(path);
    let phash = compute_perceptual_hash(path, hash_size);

    let stat = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            debug!("Cannot access file {}: {e}", path.display());
            return None;
        }
    };

    Some(ImageMeta {
        path: path.to_path_buf(),
        width,
        height,
        size: stat.len(),
        mtime: file_mtime(&stat),
        sha1,
        phash,
    })
}

/// Extract modification time as `f64` seconds since epoch.
fn file_mtime(metadata: &std::fs::Metadata) -> f64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Compute the Hamming distance between two perceptual hash byte arrays.
pub fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}
