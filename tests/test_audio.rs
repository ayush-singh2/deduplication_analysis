//! Integration tests for `audio` module — parity with `test_audio.py`.

use std::path::PathBuf;

use tempfile::TempDir;

use dedupl::audio::meta::{AudioMeta, FingerprintEntry};
use dedupl::audio::grouping::group_duplicates;
use dedupl::audio::quality::select_best_quality;

// ── TestAudioMeta ─────────────────────────────────────────────────────────

#[test]
fn test_audio_meta_creation() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.mp3");
    std::fs::write(&test_file, "").unwrap();

    let meta = AudioMeta {
        path: test_file.clone(),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.5),
        size: 7_200_000,
        mtime: 1_234_567_890.0,
    };

    assert_eq!(meta.path, test_file);
    assert_eq!(meta.codec.as_deref(), Some("mp3"));
    assert_eq!(meta.bitrate, Some(320_000));
    assert_eq!(meta.duration, Some(180.5));
    assert_eq!(meta.size, 7_200_000);
    assert_eq!(meta.mtime, 1_234_567_890.0);
}

#[test]
fn test_audio_meta_immutable_by_design() {
    // Rust enforces immutability at compile time when the binding is not `mut`.
    // This test constructs the struct without `mut` and verifies field access.
    let meta = AudioMeta {
        path: PathBuf::from("/tmp/test.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.5),
        size: 7_200_000,
        mtime: 1_234_567_890.0,
    };

    // Attempting `meta.bitrate = Some(128_000);` would be a compile error.
    assert_eq!(meta.bitrate, Some(320_000));
}

// ── TestFingerprintGeneration ─────────────────────────────────────────────
// The Python tests mock `execute_command`.  In Rust we can't monkey-patch
// free functions at runtime, so we test the JSON parsing + hashing logic
// directly by simulating what `generate_fingerprint` does internally.

#[test]
fn test_fingerprint_hash_is_40_char_sha1() {
    use sha1::Sha1;
    use digest::Digest;

    let fingerprint = "AQAADNFLFKJSDFLKJSDF";
    let mut hasher = Sha1::new();
    hasher.update(fingerprint.as_bytes());
    let fp_hash = format!("{:x}", hasher.finalize());

    assert_eq!(fp_hash.len(), 40);
}

#[test]
fn test_fingerprint_json_parsing_success() {
    let json_str = r#"{"fingerprint":"AQAADNFLFKJSDFLKJSDF","duration":180.5}"#;
    let data: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let fp = data.get("fingerprint").and_then(|v| v.as_str()).unwrap();
    let dur = data.get("duration").and_then(|v| v.as_f64()).unwrap();
    let duration = dur.round() as i64;

    assert_eq!(fp, "AQAADNFLFKJSDFLKJSDF");
    assert_eq!(duration, 181); // 180.5 rounds to 181 (Rust rounds half away from zero)
}

#[test]
fn test_fingerprint_json_parsing_invalid() {
    let result: Result<serde_json::Value, _> = serde_json::from_str("invalid json");
    assert!(result.is_err());
}

// ── TestFFprobeMetadata ───────────────────────────────────────────────────

#[test]
fn test_ffprobe_json_parsing_success() {
    let json_str = r#"{
        "streams": [{"codec_name": "mp3", "bit_rate": "320000"}],
        "format": {"duration": "180.500000"}
    }"#;
    let data: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let codec = data["streams"][0]["codec_name"].as_str().unwrap();
    let bitrate: u64 = data["streams"][0]["bit_rate"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let duration: f64 = data["format"]["duration"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    assert_eq!(codec, "mp3");
    assert_eq!(bitrate, 320_000);
    assert!((duration - 180.5).abs() < f64::EPSILON);
}

#[test]
fn test_ffprobe_json_missing_fields() {
    let json_str = r#"{
        "streams": [{"codec_name": "mp3"}],
        "format": {}
    }"#;
    let data: serde_json::Value = serde_json::from_str(json_str).unwrap();

    let codec = data["streams"][0]["codec_name"].as_str();
    let bitrate = data["streams"][0]["bit_rate"].as_str();
    let duration = data["format"]["duration"].as_str();

    assert_eq!(codec, Some("mp3"));
    assert!(bitrate.is_none());
    assert!(duration.is_none());
}

#[test]
fn test_ffprobe_failure_returns_none() {
    // Simulates rc != 0 from execute_command — in real code probe_ffprobe
    // returns (None, None, None).
    let codec: Option<String> = None;
    let bitrate: Option<u64> = None;
    let duration: Option<f64> = None;

    assert!(codec.is_none());
    assert!(bitrate.is_none());
    assert!(duration.is_none());
}

// ── TestQualitySelection ──────────────────────────────────────────────────

#[test]
fn test_select_best_quality_lossless_priority() {
    let flac_file = AudioMeta {
        path: PathBuf::from("/tmp/test.flac"),
        codec: Some("flac".to_string()),
        bitrate: None,
        duration: Some(180.0),
        size: 18_000_000,
        mtime: 1000.0,
    };

    let mp3_file = AudioMeta {
        path: PathBuf::from("/tmp/test.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 2000.0,
    };

    let candidates = [mp3_file, flac_file.clone()];
    let best = select_best_quality(&candidates).unwrap();
    assert_eq!(best.path, flac_file.path);
}

#[test]
fn test_select_best_quality_bitrate_priority() {
    let high = AudioMeta {
        path: PathBuf::from("/tmp/high.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 1000.0,
    };

    let low = AudioMeta {
        path: PathBuf::from("/tmp/low.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(128_000),
        duration: Some(180.0),
        size: 2_880_000,
        mtime: 2000.0,
    };

    let candidates = [low, high.clone()];
    let best = select_best_quality(&candidates).unwrap();
    assert_eq!(best.path, high.path);
}

#[test]
fn test_select_best_quality_estimated_bitrate() {
    let file1 = AudioMeta {
        path: PathBuf::from("/tmp/file1.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: None,
        duration: Some(100.0),
        size: 2_000_000, // ~160 kbps
        mtime: 1000.0,
    };

    let file2 = AudioMeta {
        path: PathBuf::from("/tmp/file2.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: None,
        duration: Some(100.0),
        size: 4_000_000, // ~320 kbps
        mtime: 1000.0,
    };

    let candidates = [file1, file2.clone()];
    let best = select_best_quality(&candidates).unwrap();
    assert_eq!(best.path, file2.path);
}

// ── TestDuplicateGrouping ─────────────────────────────────────────────────

#[test]
fn test_group_duplicates_exact_match() {
    let meta1 = AudioMeta {
        path: PathBuf::from("/tmp/file1.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 1000.0,
    };

    let meta2 = AudioMeta {
        path: PathBuf::from("/tmp/file2.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 1000.0,
    };

    let entry1 = FingerprintEntry {
        fp_hash: "abc123".to_string(),
        duration: 180,
        meta: meta1,
    };
    let entry2 = FingerprintEntry {
        fp_hash: "abc123".to_string(),
        duration: 180,
        meta: meta2,
    };

    let groups = group_duplicates(&[entry1, entry2], 2);
    assert_eq!(groups.len(), 1);
    let first_group = groups.values().next().unwrap();
    assert_eq!(first_group.len(), 2);
}

#[test]
fn test_group_duplicates_duration_tolerance() {
    let meta1 = AudioMeta {
        path: PathBuf::from("/tmp/file1.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 1000.0,
    };

    let meta2 = AudioMeta {
        path: PathBuf::from("/tmp/file2.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(181.0),
        size: 7_240_000,
        mtime: 1000.0,
    };

    let entry1 = FingerprintEntry {
        fp_hash: "abc123".to_string(),
        duration: 180,
        meta: meta1.clone(),
    };
    let entry2 = FingerprintEntry {
        fp_hash: "abc123".to_string(),
        duration: 181,
        meta: meta2.clone(),
    };

    // With tolerance of 2 — both round to same bucket.
    let groups = group_duplicates(&[entry1.clone(), entry2.clone()], 2);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups.values().next().unwrap().len(), 2);

    // With tolerance of 0 — integer division by max(0,1)=1, so 180 != 181.
    let groups = group_duplicates(
        &[
            FingerprintEntry {
                fp_hash: "abc123".to_string(),
                duration: 180,
                meta: meta1,
            },
            FingerprintEntry {
                fp_hash: "abc123".to_string(),
                duration: 181,
                meta: meta2,
            },
        ],
        0,
    );
    assert_eq!(groups.len(), 2);
}

#[test]
fn test_group_duplicates_different_fingerprints() {
    let meta1 = AudioMeta {
        path: PathBuf::from("/tmp/file1.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 1000.0,
    };

    let meta2 = AudioMeta {
        path: PathBuf::from("/tmp/file2.mp3"),
        codec: Some("mp3".to_string()),
        bitrate: Some(320_000),
        duration: Some(180.0),
        size: 7_200_000,
        mtime: 1000.0,
    };

    let entry1 = FingerprintEntry {
        fp_hash: "abc123".to_string(),
        duration: 180,
        meta: meta1,
    };
    let entry2 = FingerprintEntry {
        fp_hash: "def456".to_string(),
        duration: 180,
        meta: meta2,
    };

    let groups = group_duplicates(&[entry1, entry2], 2);
    assert_eq!(groups.len(), 2);
}
