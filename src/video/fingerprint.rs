// Video fingerprinting for deduplication
// Place in dedupl-rs/video/fingerprint.rs

use opencv::{core, imgproc, prelude::*, videoio};
use sha1::{Digest, Sha1};
use std::path::Path;

pub fn compute_frame_hash(frame: &core::Mat) -> String {
    let mut resized = core::Mat::default();
    imgproc::resize(
        frame,
        &mut resized,
        core::Size::new(32, 32),
        0.0,
        0.0,
        imgproc::INTER_AREA,
    )
    .unwrap();
    let mut gray = core::Mat::default();
    imgproc::cvt_color(&resized, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    let mean = core::mean(&gray, &core::no_array()).unwrap()[0];
    let mut hash_bytes = vec![];
    for y in 0..gray.rows() {
        for x in 0..gray.cols() {
            let v = *gray.at_2d::<u8>(y, x).unwrap();
            hash_bytes.push(if v as f64 > mean { 1u8 } else { 0u8 });
        }
    }
    let mut hasher = Sha1::new();
    hasher.update(&hash_bytes);
    format!("{:x}", hasher.finalize())
}

pub fn fingerprint_video(path: &Path, frame_sample_rate: usize) -> Option<Vec<String>> {
    let mut cap = videoio::VideoCapture::from_file(path.to_str()?, videoio::CAP_ANY).ok()?;
    if !cap.is_opened().ok()? {
        return None;
    }
    let frame_count = cap.get(videoio::CAP_PROP_FRAME_COUNT).ok()? as usize;
    let mut hashes = vec![];
    for i in (0..frame_count).step_by(frame_sample_rate) {
        cap.set(videoio::CAP_PROP_POS_FRAMES, i as f64).ok()?;
        let mut frame = core::Mat::default();
        if cap.read(&mut frame).ok()? && !frame.empty() {
            hashes.push(compute_frame_hash(&frame));
        }
    }
    Some(hashes)
}
