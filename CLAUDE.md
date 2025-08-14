# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**deDupl** is a Python toolkit for detecting and removing duplicate audio, image, and video files using perceptual and content-based fingerprinting. The project consists of three main standalone scripts that can be run independently.

## Core Scripts

- `dedupl_audio_fingerprints.py` - Audio deduplication using Chromaprint fingerprints
- `dedupl_images_phash.py` - Image deduplication using SHA-1 and perceptual hashing  
- `dedupl_videos_vphash.py` - Video deduplication using SHA-1 and frame sampling with perceptual hashes
- `main.py` - Placeholder main entry point (currently just prints hello message)

## Running Scripts

All scripts are designed to be run with `uv` and support dry-run mode for safe testing:

### Audio Deduplication

```bash
# Dry run (preview only)
uv run dedupl_audio_fingerprints.py "/path/to/music" --dry-run --threads 8

# Move duplicates to quarantine
uv run dedupl_audio_fingerprints.py "/path/to/music" --move-to "/path/to/quarantine" --threads 8

# Delete duplicates (use with caution)
uv run dedupl_audio_fingerprints.py "/path/to/music" --delete --threads 8
```

### Image Deduplication

```bash
# Dry run with dependencies
uv run --with pillow,imagehash,tqdm,pillow-heif dedupl_images_phash.py "/path/to/images" --dry-run --threads 8

# Move duplicates
uv run --with pillow,imagehash,tqdm,pillow-heif dedupl_images_phash.py "/path/to/images" --move-to "/path/to/quarantine" --threads 8
```

### Video Deduplication  

```bash
# Dry run with dependencies
uv run --with opencv-python-headless,pillow,imagehash,tqdm dedupl_videos_vphash.py "/path/to/videos" --dry-run --threads 8

# Move duplicates
uv run --with opencv-python-headless,pillow,imagehash,tqdm dedupl_videos_vphash.py "/path/to/videos" --move-to "/path/to/quarantine" --threads 8
```

## External Dependencies

The project requires external tools to be installed and available in PATH:

- `fpcalc` (from Chromaprint) for audio fingerprinting
- `ffprobe` (from FFmpeg) for audio/video metadata extraction

## Architecture

Each script follows a similar pattern:

1. **File Discovery** - Recursively scan directories for relevant file types
2. **Metadata Extraction** - Extract file properties and generate content hashes
3. **Fingerprint Generation** - Create perceptual signatures for content comparison
4. **Duplicate Detection** - Group files by similarity using various thresholds
5. **Action Execution** - Preview, move, or delete identified duplicates

### Key Data Structures

- `AudioMeta` - Audio file metadata (codec, bitrate, duration, etc.)
- `FingerprintEntry` - Audio fingerprint with hash and metadata  
- `ImgMeta` - Image metadata with SHA-1 and perceptual hash
- `VideoMeta` - Video metadata with majority hash and sparse frame hashes

### Threading Strategy

All scripts use `concurrent.futures.ThreadPoolExecutor` for parallel processing of files. Default thread count is CPU core count, configurable via `--threads` parameter.

## File Extensions Supported

- **Audio**: `.mp3`, `.m4a`, `.aac`, `.flac`, `.wav`, `.ogg`, `.opus`, `.wma`, `.alac`, `.ape`, `.tta`, `.aiff`, `.aif`
- **Images**: `.jpg`, `.jpeg`, `.png`, `.webp`, `.bmp`, `.gif`, `.tif`, `.tiff`, `.heic`, `.heif`  
- **Videos**: `.mp4`, `.mkv`, `.mov`, `.avi`, `.wmv`, `.flv`, `.webm`, `.m4v`, `.mpg`, `.mpeg`, `.ts`

## Safety Features

- **Dry-run mode** is the default safe operation - always test before making changes
- **Quarantine strategy** - move duplicates to a separate folder rather than deleting
- **Progress tracking** with tqdm when available
- **Graceful fallbacks** when optional dependencies are missing
