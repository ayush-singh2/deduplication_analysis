//! Integration tests for `image` module — parity with `test_image.py`.

use std::path::PathBuf;

use tempfile::TempDir;

use dedupl::image::grouping::{group_by_exact_hash, group_by_perceptual_hash};
use dedupl::image::meta::ImageMeta;
use dedupl::image::phash::{compute_perceptual_hash, hamming_distance, read_image_dimensions};
use dedupl::image::quality::select_best_quality;

// ── TestImageMeta ─────────────────────────────────────────────────────────

#[test]
fn test_image_meta_creation() {
    let meta = ImageMeta {
        path: PathBuf::from("/tmp/test.jpg"),
        width: 1920,
        height: 1080,
        size: 2_048_000,
        mtime: 1_234_567_890.0,
        sha1: "abc123def456".to_string(),
        phash: None,
    };

    assert_eq!(meta.width, 1920);
    assert_eq!(meta.height, 1080);
    assert_eq!(meta.size, 2_048_000);
    assert_eq!(meta.sha1, "abc123def456");
    assert!(meta.phash.is_none());
}

#[test]
fn test_image_meta_immutable_by_design() {
    // Rust enforces this; a non-`mut` binding cannot be modified.
    let meta = ImageMeta {
        path: PathBuf::from("/tmp/test.jpg"),
        width: 1920,
        height: 1080,
        size: 2_048_000,
        mtime: 1_234_567_890.0,
        sha1: "abc123def456".to_string(),
        phash: None,
    };
    assert_eq!(meta.width, 1920);
}

// ── TestImageDimensions ───────────────────────────────────────────────────

#[test]
fn test_read_dimensions_success() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.png");

    // Create a 100×50 red image using the `image` crate.
    let img = image::RgbImage::from_fn(100, 50, |_, _| image::Rgb([255, 0, 0]));
    img.save(&test_file).unwrap();

    let (w, h) = read_image_dimensions(&test_file);
    assert_eq!(w, 100);
    assert_eq!(h, 50);
}

#[test]
fn test_read_dimensions_failure() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("invalid.jpg");
    std::fs::write(&test_file, "not an image").unwrap();

    let (w, h) = read_image_dimensions(&test_file);
    assert_eq!(w, 0);
    assert_eq!(h, 0);
}

#[test]
fn test_read_dimensions_nonexistent() {
    let (w, h) = read_image_dimensions(std::path::Path::new("/nonexistent.jpg"));
    assert_eq!(w, 0);
    assert_eq!(h, 0);
}

// ── TestPerceptualHash ────────────────────────────────────────────────────

#[test]
fn test_compute_perceptual_hash_success() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.png");

    let img = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([0u8, 0, 255]));
    img.save(&test_file).unwrap();

    let phash = compute_perceptual_hash(&test_file, 8);
    assert!(phash.is_some());
}

#[test]
fn test_compute_perceptual_hash_different_images() {
    let tmp = TempDir::new().unwrap();

    let file1 = tmp.path().join("img1.png");
    let img1 = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([255u8, 0, 0]));
    img1.save(&file1).unwrap();

    let file2 = tmp.path().join("img2.png");
    let img2 = image::RgbImage::from_fn(100, 100, |x, y| {
        if (x % 10 == 0) && (y % 10 == 0) {
            image::Rgb([255u8, 255, 0])
        } else {
            image::Rgb([0u8, 0, 255])
        }
    });
    img2.save(&file2).unwrap();

    let hash1 = compute_perceptual_hash(&file1, 8).unwrap();
    let hash2 = compute_perceptual_hash(&file2, 8).unwrap();

    let distance = hamming_distance(&hash1, &hash2);
    assert!(distance > 0, "Different images should have different hashes");
}

#[test]
fn test_compute_perceptual_hash_similar_images() {
    let tmp = TempDir::new().unwrap();

    let file1 = tmp.path().join("img1.png");
    let img1 = image::RgbImage::from_fn(100, 100, |_, _| image::Rgb([255u8, 0, 0]));
    img1.save(&file1).unwrap();

    // Nearly identical — one pixel changed.
    let file2 = tmp.path().join("img2.png");
    let img2 = image::RgbImage::from_fn(100, 100, |x, y| {
        if x == 50 && y == 50 {
            image::Rgb([254u8, 0, 0])
        } else {
            image::Rgb([255u8, 0, 0])
        }
    });
    img2.save(&file2).unwrap();

    let hash1 = compute_perceptual_hash(&file1, 8).unwrap();
    let hash2 = compute_perceptual_hash(&file2, 8).unwrap();

    let distance = hamming_distance(&hash1, &hash2);
    assert!(distance < 5, "Similar images should have close hashes, got {distance}");
}

// ── TestQualitySelection ──────────────────────────────────────────────────

#[test]
fn test_select_best_quality_resolution_priority() {
    let high_res = ImageMeta {
        path: PathBuf::from("/tmp/high.jpg"),
        width: 3840,
        height: 2160,
        size: 4_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: None,
    };

    let low_res = ImageMeta {
        path: PathBuf::from("/tmp/low.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 2000.0,
        sha1: "def456".to_string(),
        phash: None,
    };

    let candidates = [low_res, high_res.clone()];
    let best = select_best_quality(&candidates).unwrap();
    assert_eq!(best.path, high_res.path);
}

#[test]
fn test_select_best_quality_size_tiebreaker() {
    let file1 = ImageMeta {
        path: PathBuf::from("/tmp/file1.jpg"),
        width: 1920,
        height: 1080,
        size: 3_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: None,
    };

    let file2 = ImageMeta {
        path: PathBuf::from("/tmp/file2.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "def456".to_string(),
        phash: None,
    };

    let candidates = [file1.clone(), file2];
    let best = select_best_quality(&candidates).unwrap();
    assert_eq!(best.path, file1.path);
}

#[test]
fn test_select_best_quality_mtime_tiebreaker() {
    let file1 = ImageMeta {
        path: PathBuf::from("/tmp/file1.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 2000.0,
        sha1: "abc123".to_string(),
        phash: None,
    };

    let file2 = ImageMeta {
        path: PathBuf::from("/tmp/file2.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "def456".to_string(),
        phash: None,
    };

    let candidates = [file1.clone(), file2];
    let best = select_best_quality(&candidates).unwrap();
    assert_eq!(best.path, file1.path);
}

// ── TestExactHashGrouping ─────────────────────────────────────────────────

#[test]
fn test_group_by_exact_hash_identical() {
    let meta1 = ImageMeta {
        path: PathBuf::from("/tmp/file1.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: None,
    };

    let meta2 = ImageMeta {
        path: PathBuf::from("/tmp/file2.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(), // Same hash
        phash: None,
    };

    let groups = group_by_exact_hash(&[meta1, meta2]);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 2);
}

#[test]
fn test_group_by_exact_hash_different() {
    let meta1 = ImageMeta {
        path: PathBuf::from("/tmp/file1.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: None,
    };

    let meta2 = ImageMeta {
        path: PathBuf::from("/tmp/file2.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "def456".to_string(), // Different hash
        phash: None,
    };

    let groups = group_by_exact_hash(&[meta1, meta2]);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_group_by_exact_hash_multiple_groups() {
    let items: Vec<ImageMeta> = (0..4)
        .map(|i| ImageMeta {
            path: PathBuf::from(format!("/tmp/file{i}.jpg")),
            width: 100,
            height: 100,
            size: 1000,
            mtime: 1000.0,
            sha1: if i < 2 {
                "hash1".to_string()
            } else {
                "hash2".to_string()
            },
            phash: None,
        })
        .collect();

    let groups = group_by_exact_hash(&items);
    assert_eq!(groups.len(), 2);
    assert!(groups.iter().all(|g| g.len() == 2));
}

// ── TestPerceptualHashGrouping ────────────────────────────────────────────

#[test]
fn test_group_by_perceptual_hash_similar() {
    // Two hashes with small Hamming distance (identical bytes = distance 0).
    let hash_bytes = vec![0xAA, 0xBB, 0xCC, 0xDD];

    let meta1 = ImageMeta {
        path: PathBuf::from("/tmp/file1.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: Some(hash_bytes.clone()),
    };

    let meta2 = ImageMeta {
        path: PathBuf::from("/tmp/file2.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "def456".to_string(),
        phash: Some(hash_bytes), // Same hash → distance 0
    };

    let groups = group_by_perceptual_hash(&[meta1, meta2], 5, false);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 2);
}

#[test]
fn test_group_by_perceptual_hash_different() {
    // Two hashes with large Hamming distance.
    let hash1 = vec![0x00, 0x00, 0x00, 0x00];
    let hash2 = vec![0xFF, 0xFF, 0xFF, 0xFF]; // distance = 32

    let meta1 = ImageMeta {
        path: PathBuf::from("/tmp/file1.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: Some(hash1),
    };

    let meta2 = ImageMeta {
        path: PathBuf::from("/tmp/file2.jpg"),
        width: 1920,
        height: 1080,
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "def456".to_string(),
        phash: Some(hash2),
    };

    let groups = group_by_perceptual_hash(&[meta1, meta2], 5, false);
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_group_by_perceptual_hash_aspect_ratio_check() {
    // Same hash but different aspect ratios.
    let hash_bytes = vec![0xAA, 0xBB, 0xCC, 0xDD];

    let meta_wide = ImageMeta {
        path: PathBuf::from("/tmp/wide.jpg"),
        width: 1920,
        height: 1080, // 16:9
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "abc123".to_string(),
        phash: Some(hash_bytes.clone()),
    };

    let meta_square = ImageMeta {
        path: PathBuf::from("/tmp/square.jpg"),
        width: 1000,
        height: 1000, // 1:1
        size: 2_000_000,
        mtime: 1000.0,
        sha1: "def456".to_string(),
        phash: Some(hash_bytes),
    };

    // With aspect ratio check: should NOT group (1.78 vs 1.0).
    let groups =
        group_by_perceptual_hash(&[meta_wide.clone(), meta_square.clone()], 5, true);
    assert_eq!(groups.len(), 0);

    // Without aspect ratio check: should group.
    let groups = group_by_perceptual_hash(&[meta_wide, meta_square], 5, false);
    assert_eq!(groups.len(), 1);
}
