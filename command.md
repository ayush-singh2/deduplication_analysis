# Audio files deduplicate

### Run it with `uv`

Dry run:

```bash
uv run dedupe_audio_fingerprints.py "/path/to/Music" --threads 8 --dry-run
```

Move duplicates to a quarantine folder:

```bash
uv run dedupe_audio_fingerprints.py "/path/to/Music" --threads 8 --move-to "/path/to/Quarantine"
```

Delete duplicates (only if sure):

```bash
uv run dedupe_audio_fingerprints.py "/path/to/Music" --threads 8 --delete
```


### Run with uv

Dry run (with progress bars):

```bash
uv run --with tqdm dedupe_audio_fingerprints.py "/path/to/Music" --threads 8 --dry-run
```

Move duplicates:

```bash
uv run --with tqdm dedupe_audio_fingerprints.py "/path/to/Music" --threads 8 --move-to "/path/to/Quarantine"
```

---

# Images deduplicate
## Run it with `uv`

### 1) Install deps (once via `uv`)

```bash
# Add progress bars + HEIC support
uv run -q --with pillow,imagehash,tqdm,pillow-heif -c "import PIL, imagehash, tqdm; print('ok')"
```

### 2) Dry run (no changes)

```bash
uv run --with pillow,imagehash,tqdm,pillow-heif \
  dedupe_images_phash.py "/path/to/Photos" \
  --threads 8 --dry-run
```

### 3) Move duplicates to Quarantine

```bash
uv run --with pillow,imagehash,tqdm,pillow-heif \
  dedupe_images_phash.py "/path/to/Photos" \
  --threads 8 --move-to "/path/to/Quarantine"
```

### 4) Permanently delete (only if sure)

```bash
uv run --with pillow,imagehash,tqdm,pillow-heif \
  dedupe_images_phash.py "/path/to/Photos" \
  --threads 8 --delete
```



# Video deduplicate
Run it with uv on Debian
Install Python deps once (no system packages needed):


```
uv run -q --with opencv-python-headless,pillow,imagehash,tqdm -c "import cv2, PIL, imagehash, tqdm; print('ok')"
```

## 1) Dry run (no changes):

```bash
uv run --with opencv-python-headless,pillow,imagehash,tqdm \
  dedupe_videos_vphash.py "/path/to/Videos" \
  --threads 8 --dry-run
```

## 2) Move duplicates to Quarantine (safer):
```bash
uv run --with opencv-python-headless,pillow,imagehash,tqdm \
  dedupe_videos_vphash.py "/path/to/Videos" \
  --threads 8 --move-to "/path/to/Quarantine"
```

## 3) Delete duplicates (only when sure):
```bash
uv run --with opencv-python-headless,pillow,imagehash,tqdm \
  dedupe_videos_vphash.py "/path/to/Videos" \
  --threads 8 --delete
```