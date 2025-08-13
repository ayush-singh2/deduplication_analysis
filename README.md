# deDupl

**deDupl** is a Python toolkit for detecting and removing duplicate audio, image, and video files using perceptual and content-based fingerprinting. It helps you free up disk space and organize your media library efficiently.

## Features

- Scans directories for duplicate audio, image, and video files based on content (not just filename or metadata)
- Uses Chromaprint (`fpcalc`) and `ffprobe` for audio, perceptual hashes for images and videos
- Supports recursive directory scanning
- Provides options to preview, move, or delete duplicates
- Fast and efficient comparison using hashing and multithreading

## Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/yourusername/deDupl.git
   ```
2. Navigate to the project directory:
   ```sh
   cd deDupl
   ```
3. Install Python dependencies using [uv](https://github.com/astral-sh/uv):
   ```sh
   uv pip install -r requirements.txt
   ```
4. Ensure you have the following external tools installed and available in your PATH:
   - `fpcalc` (from Chromaprint) for audio deduplication
   - `ffprobe` (from FFmpeg) for audio/video metadata
   - `opencv-python`, `Pillow`, `imagehash`, `tqdm`, and optionally `pillow-heif` for image/video deduplication

## Usage

See [commands.md](./commands.md) for detailed command-line usage and examples for audio, image, and video deduplication.

**Typical usage pattern:**
- To preview duplicates:  
  ```sh
  uv run dedupl_audio_fingerprints.py /path/to/music --dry-run
  ```
- To move or delete duplicates, add `--move-to` or `--delete` as described in [commands.md](./commands.md).

### Audio Deduplication

Script: `dedupl_audio_fingerprints.py`

Detect and remove duplicate audio files using Chromaprint fingerprints.

**Basic scan (preview only, no changes made):**
```sh
python dedupl_audio_fingerprints.py /path/to/music --dry-run
```

**Move duplicates to a folder:**
```sh
python dedupl_audio_fingerprints.py /path/to/music --move-to /path/to/duplicates_folder
```

**Delete duplicates:**
```sh
python dedupl_audio_fingerprints.py /path/to/music --delete
```

**Additional options:**
- `--threads N` : Number of threads to use (default: number of CPU cores)
- `--min-duration SECONDS` : Ignore files shorter than this (default: 20)
- `--same-duration-tolerance N` : Tolerance in seconds for considering durations as "same" (default: 2)
- `--dry-run` : Preview actions only (no files are moved or deleted)

#### Example:

```sh
python dedupl_audio_fingerprints.py ~/Music --move-to ~/Music/duplicates --min-duration 30 --threads 8
```

---

### Image Deduplication

Script: `dedupl_images_phash.py`

Detect and remove duplicate images using exact SHA-1 and perceptual pHash.

**Basic scan (preview only, no changes made):**
```sh
python dedupl_images_phash.py /path/to/images --dry-run
```

**Move duplicates to a folder:**
```sh
python dedupl_images_phash.py /path/to/images --move-to /path/to/duplicates_folder
```

**Delete duplicates:**
```sh
python dedupl_images_phash.py /path/to/images --delete
```

**Additional options:**
- `--threads N` : Number of threads to use (default: number of CPU cores)
- `--hash-size N` : pHash size (default: 16)
- `--threshold N` : Max Hamming distance for pHash duplicates (default: 6)
- `--prefix-bits N` : Bucket by first N bits of pHash (default: 18)
- `--min-pixels N` : Ignore images smaller than this (default: 25000)
- `--dry-run` : Preview actions only (no files are moved or deleted)

---

### Video Deduplication

Script: `dedupl_videos_vphash.py`

Detect and remove duplicate videos using exact SHA-1 and perceptual hashes over sampled frames.

**Basic scan (preview only, no changes made):**
```sh
python dedupl_videos_vphash.py /path/to/videos --dry-run
```

**Move duplicates to a folder:**
```sh
python dedupl_videos_vphash.py /path/to/videos --move-to /path/to/duplicates_folder
```

**Delete duplicates:**
```sh
python dedupl_videos_vphash.py /path/to/videos --delete
```

**Additional options:**
- `--threads N` : Number of threads to use (default: number of CPU cores)
- `--fps-sample F` : Frames per second to sample (default: 0.5)
- `--max-secs N` : Max seconds to sample per video (default: 240)
- `--hash-size N` : pHash size (default: 8)
- `--majority-threshold N` : Max Hamming distance for majority-hash (default: 8)
- `--avg-threshold F` : Max average Hamming over sampled frames (default: 10.0)
- `--duration-tol F` : Duration tolerance for grouping (default: 3.0)
- `--dry-run` : Preview actions only (no files are moved or deleted)

---

## Requirements

- Python 3.7+
- [uv](https://github.com/astral-sh/uv) for dependency and script management
- [Chromaprint](https://acoustid.org/chromaprint) (`fpcalc` command-line tool) for audio
- [FFmpeg](https://ffmpeg.org/) (`ffprobe` command-line tool) for audio/video
- [OpenCV](https://pypi.org/project/opencv-python/) (`opencv-python`) for video
- [Pillow](https://pypi.org/project/Pillow/) and [imagehash](https://pypi.org/project/ImageHash/) for images/videos
- tqdm (optional, for progress bars)
- pillow-heif (optional, for HEIC/HEIF image support)

Install Python dependencies with:

```sh
uv pip install -r requirements.txt
```

## Contributing

Contributions are welcome! Please open issues or submit pull requests for improvements.

## License

This project is licensed under the MIT License.
