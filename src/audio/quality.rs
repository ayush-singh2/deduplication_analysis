//! Audio quality selection — mirrors Python's `select_best_quality`.

use super::meta::{AudioMeta, LOSSLESS_CODECS};

/// Select the best quality audio file from a list of duplicates.
///
/// Priority order:
/// 1. Lossless codec
/// 2. Higher bitrate (or estimated from file size / duration)
/// 3. Larger file size
/// 4. Newer modification time
/// 5. Shorter path (likely the original location)
pub fn select_best_quality(candidates: &[AudioMeta]) -> Option<&AudioMeta> {
    if candidates.is_empty() {
        return None;
    }

    candidates.iter().min_by(|a, b| {
        let a_lossless = is_lossless(a.codec.as_deref());
        let b_lossless = is_lossless(b.codec.as_deref());

        // Higher lossless first (reverse ordering).
        b_lossless
            .cmp(&a_lossless)
            .then_with(|| {
                let a_br = bitrate_or_estimate(a);
                let b_br = bitrate_or_estimate(b);
                b_br.cmp(&a_br)
            })
            .then_with(|| b.size.cmp(&a.size))
            .then_with(|| {
                b.mtime
                    .partial_cmp(&a.mtime)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                let a_len = a.path.to_string_lossy().len();
                let b_len = b.path.to_string_lossy().len();
                a_len.cmp(&b_len)
            })
    })
}

fn is_lossless(codec: Option<&str>) -> u8 {
    match codec {
        Some(c) if LOSSLESS_CODECS.contains(&c.to_lowercase().as_str()) => 1,
        _ => 0,
    }
}

fn bitrate_or_estimate(meta: &AudioMeta) -> u64 {
    if let Some(br) = meta.bitrate {
        if br > 0 {
            return br;
        }
    }
    if let Some(dur) = meta.duration {
        if dur > 0.0 {
            return ((meta.size as f64 / dur) * 8.0) as u64;
        }
    }
    0
}
