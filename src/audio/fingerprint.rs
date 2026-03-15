//! Audio fingerprint generation and ffprobe metadata extraction.

use std::path::Path;

use sha1::Sha1;
use digest::Digest;
use tracing::debug;

use crate::common::command::execute_command;
use crate::common::security::validate_path_security;

use super::meta::{AudioMeta, FingerprintEntry};

/// Extract audio metadata using `ffprobe`.
///
/// Returns `(codec_name, bitrate_bps, duration_seconds)`.
pub fn probe_ffprobe(path: &Path) -> (Option<String>, Option<u64>, Option<f64>) {
    if !validate_path_security(path) {
        return (None, None, None);
    }

    let path_str = path.to_string_lossy();
    let cmd: Vec<&str> = vec![
        "ffprobe",
        "-v",
        "error",
        "-select_streams",
        "a:0",
        "-show_entries",
        "stream=codec_name,bit_rate:format=duration",
        "-of",
        "json",
        &path_str,
    ];

    let (rc, out, err) = execute_command(&cmd, Some(std::time::Duration::from_secs(30)));
    if rc != 0 {
        debug!("ffprobe failed for {}: {err}", path.display());
        return (None, None, None);
    }

    let data: serde_json::Value = match serde_json::from_str(&out) {
        Ok(v) => v,
        Err(e) => {
            debug!("Failed to parse ffprobe output for {}: {e}", path.display());
            return (None, None, None);
        }
    };

    let mut codec: Option<String> = None;
    let mut bitrate: Option<u64> = None;
    let mut duration: Option<f64> = None;

    if let Some(streams) = data.get("streams").and_then(|s| s.as_array()) {
        if let Some(stream) = streams.first() {
            codec = stream
                .get("codec_name")
                .and_then(|v| v.as_str())
                .map(String::from);

            bitrate = stream
                .get("bit_rate")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok());
        }
    }

    if let Some(format) = data.get("format") {
        duration = format
            .get("duration")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok());
    }

    (codec, bitrate, duration)
}

/// Generate an audio fingerprint using Chromaprint's `fpcalc`.
///
/// Returns `(sha1_of_fingerprint, rounded_duration)` or `None` on error.
pub fn generate_fingerprint(path: &Path) -> Option<(String, i64)> {
    if !validate_path_security(path) {
        return None;
    }

    let path_str = path.to_string_lossy();
    let cmd: Vec<&str> = vec!["fpcalc", "-json", &path_str];

    let (rc, out, err) = execute_command(&cmd, Some(std::time::Duration::from_secs(60)));
    if rc != 0 {
        debug!("fpcalc failed for {}: {err}", path.display());
        return None;
    }

    let data: serde_json::Value = match serde_json::from_str(&out) {
        Ok(v) => v,
        Err(e) => {
            debug!("Failed to parse fpcalc output for {}: {e}", path.display());
            return None;
        }
    };

    let fingerprint = data.get("fingerprint")?.as_str()?;
    let dur_raw = data.get("duration")?.as_f64()?;
    let duration = dur_raw.round() as i64;

    if fingerprint.is_empty() || duration <= 0 {
        debug!("Invalid fingerprint data for {}", path.display());
        return None;
    }

    // SHA-1 hash of the fingerprint string.
    let mut hasher = Sha1::new();
    hasher.update(fingerprint.as_bytes());
    let fp_hash = format!("{:x}", hasher.finalize());

    Some((fp_hash, duration))
}

/// Scan an audio file: generate fingerprint + extract metadata.
pub fn scan_audio_file(path: &Path) -> Option<FingerprintEntry> {
    let (fp_hash, duration_fp) = generate_fingerprint(path)?;
    let (codec, bitrate, duration_probe) = probe_ffprobe(path);

    let stat = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            debug!("Cannot access file {}: {e}", path.display());
            return None;
        }
    };

    let duration = duration_probe.or(Some(duration_fp as f64));

    let meta = AudioMeta {
        path: path.to_path_buf(),
        codec,
        bitrate,
        duration,
        size: stat.len(),
        mtime: file_mtime(&stat),
    };

    Some(FingerprintEntry {
        fp_hash,
        duration: duration_fp,
        meta,
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
