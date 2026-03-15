//! Image quality selection — mirrors Python's `select_best_quality`.

use super::meta::ImageMeta;

/// Select the best quality image from a list of duplicates.
///
/// Priority order:
/// 1. Higher resolution (width * height)
/// 2. Larger file size
/// 3. Newer modification time
/// 4. Shorter path
pub fn select_best_quality(candidates: &[ImageMeta]) -> Option<&ImageMeta> {
    if candidates.is_empty() {
        return None;
    }

    candidates.iter().min_by(|a, b| {
        let a_res = a.resolution();
        let b_res = b.resolution();

        b_res
            .cmp(&a_res)
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
