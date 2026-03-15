// Video metadata extraction for deduplication
// Place in dedupl-rs/video/meta.rs

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct VideoMeta {
    pub path: String,
    pub codec: Option<String>,
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub size: u64,
}

impl VideoMeta {
    pub fn from_path(path: &Path) -> Option<Self> {
        let meta = std::fs::metadata(path).ok()?;
        let size = meta.len();
        let ffprobe = Command::new("ffprobe")
            .args(&["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=codec_name,width,height:format=duration", "-of", "json", path.to_str()?])
            .output()
            .ok()?;
        let out = String::from_utf8_lossy(&ffprobe.stdout);
        let v: serde_json::Value = serde_json::from_str(&out).ok()?;
        let (codec, width, height, duration) = if let Some(stream) = v["streams"].get(0) {
            (
                stream["codec_name"].as_str().map(|s| s.to_string()),
                stream["width"].as_u64().map(|w| w as u32),
                stream["height"].as_u64().map(|h| h as u32),
                v["format"]["duration"].as_str().and_then(|d| d.parse().ok()),
            )
        } else {
            (None, None, None, None)
        };
        Some(Self {
            path: path.to_string_lossy().to_string(),
            codec,
            duration,
            width,
            height,
            size,
        })
    }
}
