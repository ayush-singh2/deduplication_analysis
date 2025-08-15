# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

The project uses modern Python packaging:

- Source code in `src/dedupl/` package
- Comprehensive test suite in `tests/`

## Project Overview

**deDupl** is a Python toolkit for detecting and removing duplicate audio, image, and video files using perceptual and content-based fingerprinting. The project provides modular functionality through a clean package structure.

## Source Modules

- `src/dedupl/common.py` - Shared utilities for all deduplication types
- `src/dedupl/audio.py` - Audio deduplication logic and fingerprinting
- `src/dedupl/image.py` - Image deduplication with perceptual hashing

## Development Environment

### Setup Development Environment

```bash
make install-dev    # Install with development dependencies
make check-deps     # Verify external dependencies
```

### Running Tests

```bash
make test          # Run all tests
make test-unit     # Unit tests only
make test-coverage # With coverage report
make lint          # Code quality checks
```

### Usage

Use the modular components from the `src/dedupl/` package to build custom deduplication workflows or create CLI scripts as needed.

## External Dependencies

The project requires external tools to be installed and available in PATH:

- `fpcalc` (from Chromaprint) for audio fingerprinting
- `ffprobe` (from FFmpeg) for audio/video metadata extraction

## Architecture

The deduplication modules follow a similar pattern:

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

### Architecture Patterns

- **Modular design** with separation of concerns
- **Security-first approach** with comprehensive validation
- **Comprehensive error handling** with detailed logging
- **Parallel processing** using ThreadPoolExecutor
- **Type safety** with complete annotations
- **Testability** with dependency injection and mocking

## File Extensions Supported

- **Audio**: `.mp3`, `.m4a`, `.aac`, `.flac`, `.wav`, `.ogg`, `.opus`, `.wma`, `.alac`, `.ape`, `.tta`, `.aiff`, `.aif`
- **Images**: `.jpg`, `.jpeg`, `.png`, `.webp`, `.bmp`, `.gif`, `.tif`, `.tiff`, `.heic`, `.heif`  
- **Videos**: `.mp4`, `.mkv`, `.mov`, `.avi`, `.wmv`, `.flv`, `.webm`, `.m4v`, `.mpg`, `.mpeg`, `.ts`

## Safety & Security Features

- **Comprehensive security validation** - path traversal prevention, command injection protection
- **Dry-run mode** as default safe operation
- **Quarantine strategy** for safe duplicate handling
- **Extensive error handling** with detailed logging
- **Type safety** with complete type hints and mypy checking
- **Comprehensive test coverage** with unit tests for all modules

## Testing

### Unit Tests

- `tests/unit/test_common.py` - 40+ tests for shared utilities
- `tests/unit/test_audio.py` - 25+ tests for audio functionality
- `tests/unit/test_image.py` - 30+ tests for image functionality

### Test Features

- Mocked external dependencies (ffprobe, fpcalc)
- Security validation testing
- Error condition coverage
- Performance and quality algorithm testing

## Dependencies

### Python Packages

- Core: tqdm (progress bars)
- Audio: No additional Python deps (uses external fpcalc)
- Images: Pillow, imagehash, pillow-heif
- Video: opencv-python-headless, numpy
- Dev: pytest, black, ruff, mypy, coverage

### External Tools

- **fpcalc** (Chromaprint): Audio fingerprinting
- **ffprobe** (FFmpeg): Audio/video metadata

Use `make check-deps` to verify all dependencies.
