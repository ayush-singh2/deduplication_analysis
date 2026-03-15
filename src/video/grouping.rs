// Video duplicate grouping for deduplication
// Place in dedupl-rs/video/grouping.rs

use std::collections::HashMap;
use crate::video::meta::VideoMeta;

pub fn group_video_duplicates(
    metas: &[VideoMeta],
    fingerprints: &[Vec<String>],
    frame_match_threshold: f32,
) -> HashMap<usize, Vec<usize>> {
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut group_id = 0;
    let mut assigned = vec![false; metas.len()];
    for (i, fp1) in fingerprints.iter().enumerate() {
        if assigned[i] {
            continue;
        }
        let mut group = vec![i];
        assigned[i] = true;
        for (j, fp2) in fingerprints.iter().enumerate().skip(i + 1) {
            if assigned[j] {
                continue;
            }
            let overlap = fp1.iter().filter(|h| fp2.contains(h)).count();
            let min_len = fp1.len().min(fp2.len());
            if min_len > 0 && (overlap as f32 / min_len as f32) >= frame_match_threshold {
                group.push(j);
                assigned[j] = true;
            }
        }
        groups.insert(group_id, group);
        group_id += 1;
    }
    groups
}
