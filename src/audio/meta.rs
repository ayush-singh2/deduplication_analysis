//! Audio metadata structs — mirrors Python's `AudioMeta` and `FingerprintEntry`.

use std::path::PathBuf;

/// Supported audio file extensions (lowercase, with leading dot).
pub const AUDIO_EXTS: &[&str] = &[
    ".mp3", ".m4a", ".aac", ".flac", ".wav", ".ogg", ".opus", ".wma", ".alac", ".ape", ".tta",
    ".aiff", ".aif",
];

/// Lossless audio codecs used for quality prioritisation.
pub const LOSSLESS_CODECS: &[&str] = &[
    "flac",
    "wav",
    "alac",
    "ape",
    "tta",
    "pcm_s16le",
    "pcm_s24le",
    "aiff",
];

/// Metadata for a single audio file.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioMeta {
    pub path: PathBuf,
    pub codec: Option<String>,
    pub bitrate: Option<u64>,
    pub duration: Option<f64>,
    pub size: u64,
    pub mtime: f64,
}

/// Audio fingerprint entry with associated metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct FingerprintEntry {
    pub fp_hash: String,
    pub duration: i64,
    pub meta: AudioMeta,
}
