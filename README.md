
# dedupl-rs

High-performance Rust toolkit for deduplication of audio, image, and video files using perceptual and content-based fingerprinting.

## Features

- Fast audio fingerprinting (Chromaprint integration)
- Perceptual hashing for images and videos
- Secure file operations and metadata extraction
- Parallel processing for efficient scanning
- Modular, extensible architecture
- Comprehensive error handling and logging

## 📁 Project Structure

```text
dedupl-rs/
│
├── src/
│   ├── audio/
│   │   ├── fingerprint.rs
│   │   ├── grouping.rs
│   │   └── quality.rs
│   │
│   ├── image/
│   │   ├── hashing.rs
│   │   ├── grouping.rs
│   │   └── quality.rs
│   │
│   ├── video/
│   │   ├── frame_hash.rs
│   │   ├── grouping.rs
│   │   └── metadata.rs
│   │
│   └── common/
│       ├── command.rs
│       ├── config.rs
│       ├── filesystem.rs
│       ├── security.rs
│       └── stats.rs
│
├── tests/
│   └── integration_tests.rs
│
├── Cargo.toml
└── README.md
```

## Supported File Types

- **Audio:** mp3, m4a, aac, flac, wav, ogg, opus, wma, alac, ape, tta, aiff, aif
- **Images:** jpg, jpeg, png, webp, bmp, gif, tif, tiff, heic, heif
- **Videos:** mp4, mkv, mov, avi, wmv, flv, webm, m4v, mpg, mpeg, ts

## Build & Run

\`\`\`bash
cargo build --release
cargo test
cargo run --release
\`\`\`

## Extending

- Add new deduplication algorithms or file types by creating new modules under `src/` and updating `mod.rs` files.
- Follow modular patterns for separation of concerns and security.

## Development

- Error handling and logging are built-in for robust operation.
- Parallel processing is used for fast file scanning and comparison.

## License

MIT License
EOF
