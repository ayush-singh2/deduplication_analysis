#!/usr/bin/env python3
import argparse
import concurrent.futures as futures
import hashlib
import os
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Iterable

# Progress
try:
    from tqdm import tqdm
except ImportError:
    tqdm = lambda x, **k: x  # fallback if tqdm not installed

# Imaging
from PIL import Image, ImageOps
try:
    # Optional: enable HEIC/HEIF if pillow-heif is available
    from pillow_heif import register_heif_opener
    register_heif_opener()
except Exception:
    pass

try:
    import imagehash
except ImportError as e:
    raise SystemExit("Missing dependency: imagehash. Run with: uv run --with pillow,imagehash,tqdm,pillow-heif ...") from e

# Common formats; add RAW if you want (may need extra decoders)
IMAGE_EXTS = {
    ".jpg", ".jpeg", ".png", ".webp", ".bmp", ".gif", ".tif", ".tiff",
    ".heic", ".heif"
    # RAW examples (decode support varies): ".cr2", ".nef", ".arw", ".dng"
}

@dataclass(frozen=True)
class ImgMeta:
    path: Path
    width: int
    height: int
    size: int          # bytes
    mtime: float       # modification time
    sha1: str          # file hash
    phash: Optional[imagehash.ImageHash]  # may be None for unreadable images

def is_image_file(p: Path) -> bool:
    return p.suffix.lower() in IMAGE_EXTS

def file_sha1(path: Path, chunk: int = 1024 * 1024) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        while True:
            b = f.read(chunk)
            if not b:
                break
            h.update(b)
    return h.hexdigest()

def compute_phash(path: Path, hash_size: int = 16) -> Optional[imagehash.ImageHash]:
    try:
        with Image.open(path) as im:
            # Normalize orientation based on EXIF
            im = ImageOps.exif_transpose(im)
            # Convert to RGB to avoid mode issues
            im = im.convert("RGB")
            return imagehash.phash(im, hash_size=hash_size)
    except Exception:
        return None

def read_dims(path: Path) -> Tuple[int, int]:
    try:
        with Image.open(path) as im:
            im = ImageOps.exif_transpose(im)
            return im.width, im.height
    except Exception:
        return (0, 0)

def scan_one(path: Path, hash_size: int) -> Optional[ImgMeta]:
    try:
        sha1 = file_sha1(path)
        w, h = read_dims(path)
        ph = compute_phash(path, hash_size=hash_size)
        st = path.stat()
        return ImgMeta(
            path=path, width=w, height=h, size=st.st_size, mtime=st.st_mtime, sha1=sha1, phash=ph
        )
    except Exception:
        return None

def walk_images(root: Path) -> List[Path]:
    out = []
    for dp, _, fns in os.walk(root):
        for name in fns:
            p = Path(dp) / name
            if is_image_file(p):
                out.append(p)
    return out

def best_candidate(cands: List[ImgMeta]) -> ImgMeta:
    # Prefer: higher resolution (w*h) > larger size > newer mtime > shorter path
    return sorted(
        cands,
        key=lambda m: (
            -(m.width * m.height),
            -m.size,
            -m.mtime,
            len(str(m.path))
        ),
    )[0]

def bucket_by_prefix(items: Iterable[ImgMeta], prefix_bits: int, hash_size: int) -> Dict[int, List[ImgMeta]]:
    """
    Bucket pHashes by their top prefix_bits to avoid O(n^2) all-pairs.
    """
    buckets: Dict[int, List[ImgMeta]] = {}
    if prefix_bits <= 0:
        buckets[0] = list(items)
        return buckets

    for m in items:
        if m.phash is None:
            buckets.setdefault(-1, []).append(m)  # unreadable -> separate bucket
            continue
        # imagehash stores hash as numpy array underneath; use .hash to get bool array
        # Convert to int and take prefix_bits
        # We rebuild an integer from the bits
        bits = m.phash.hash.flatten()
        val = 0
        for i in range(min(prefix_bits, bits.size)):
            val = (val << 1) | int(bool(bits[i]))
        buckets.setdefault(val, []).append(m)
    return buckets

def group_by_exact_sha(items: List[ImgMeta]) -> List[List[ImgMeta]]:
    groups: Dict[str, List[ImgMeta]] = {}
    for m in items:
        groups.setdefault(m.sha1, []).append(m)
    return [v for v in groups.values() if len(v) > 1]

def group_by_phash(items: List[ImgMeta], threshold: int, hash_size: int, prefix_bits: int) -> List[List[ImgMeta]]:
    """
    Group images whose pHash Hamming distance <= threshold.
    Uses prefix bucketing to reduce comparisons.
    """
    # Filter to those with phash
    with_phash = [m for m in items if m.phash is not None]
    buckets = bucket_by_prefix(with_phash, prefix_bits=prefix_bits, hash_size=hash_size)

    visited: set[Path] = set()
    groups: List[List[ImgMeta]] = []

    for _, bucket in buckets.items():
        # Simple greedy grouping inside each bucket
        bucket_sorted = sorted(bucket, key=lambda m: (-(m.width*m.height), -m.size, -m.mtime))
        for i, base in enumerate(bucket_sorted):
            if base.path in visited:
                continue
            group = [base]
            visited.add(base.path)
            for cand in bucket_sorted[i+1:]:
                if cand.path in visited:
                    continue
                dist = base.phash - cand.phash  # Hamming distance
                # Quick guard: also ensure aspect ratio close (avoid false-positive heavy crops)
                ar_base = base.width / base.height if base.height else 0
                ar_cand = cand.width / cand.height if cand.height else 0
                ar_ok = (ar_base == 0 and ar_cand == 0) or (0.9 <= (ar_base / ar_cand if ar_cand else 0) <= 1.11)
                if dist <= threshold and ar_ok:
                    group.append(cand)
                    visited.add(cand.path)
            if len(group) > 1:
                groups.append(group)
    return groups

def human_mb(b: int) -> str:
    return f"{b / (1024*1024):.2f} MB"

def main():
    ap = argparse.ArgumentParser(description="Deduplicate images using exact SHA-1 and perceptual pHash.")
    ap.add_argument("image_dir", help="Root folder to scan")
    ap.add_argument("--threads", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--dry-run", action="store_true", help="Preview only; do not move/delete")
    ap.add_argument("--move-to", type=str, default=None, help="Quarantine folder to move duplicates")
    ap.add_argument("--delete", action="store_true", help="Delete duplicates instead of moving")
    ap.add_argument("--hash-size", type=int, default=16, help="pHash size (higher = more precise, slower)")
    ap.add_argument("--threshold", type=int, default=6, help="Max Hamming distance to treat as duplicate (0-64 for hash-size=8; 0-256 for 16)")
    ap.add_argument("--prefix-bits", type=int, default=18, help="Bucket by first N bits of pHash to speed up grouping")
    ap.add_argument("--min-pixels", type=int, default=25_000, help="Ignore tiny images (< pixels), e.g., < 250x100")
    args = ap.parse_args()

    if args.delete and args.move_to:
        print("Choose either --delete or --move-to, not both.")
        return

    root = Path(args.image_dir).expanduser().resolve()
    if not root.exists():
        print(f"Path not found: {root}")
        return

    qdir = None
    if args.move_to:
        qdir = Path(args.move_to).expanduser().resolve()
        qdir.mkdir(parents=True, exist_ok=True)

    files = walk_images(root)
    print(f"Scanning {len(files)} image files with {args.threads} threads...")

    metas: List[ImgMeta] = []
    with futures.ThreadPoolExecutor(max_workers=args.threads) as ex:
        for m in tqdm(ex.map(lambda p: scan_one(p, args.hash_size), files), total=len(files), desc="Hashing & Analyzing", unit="img"):
            if not m:
                continue
            if (m.width * m.height) < args.min_pixels:
                continue
            metas.append(m)

    # Stage 1: exact duplicates by file SHA-1
    sha_groups = group_by_exact_sha(metas)
    exact_actions = []
    exact_space = 0
    for group in sha_groups:
        keep = best_candidate(group)
        for g in group:
            if g is keep:
                continue
            exact_actions.append((g, keep))
            exact_space += g.size

    # Remove exact dupes from further (pHash) processing
    exact_dupe_paths = {g.path for g, _ in exact_actions}
    remaining = [m for m in metas if m.path not in exact_dupe_paths]

    # Stage 2: perceptual duplicates by pHash
    phash_groups = group_by_phash(remaining, threshold=args.threshold, hash_size=args.hash_size, prefix_bits=args.prefix_bits)
    phash_actions = []
    phash_space = 0
    for group in phash_groups:
        keep = best_candidate(group)
        for g in group:
            if g is keep:
                continue
            phash_actions.append((g, keep))
            phash_space += g.size

    total_actions = exact_actions + phash_actions
    total_space = exact_space + phash_space

    print(f"\nExact-duplicate groups: {len(sha_groups)} | Perceptual-duplicate groups: {len(phash_groups)}")
    print(f"Files to remove/move: {len(total_actions)} | Potential space to free: {human_mb(total_space)}\n")

    for dup, keep in total_actions:
        print(f"DUP  : {dup.path}  [{dup.width}x{dup.height}, {human_mb(dup.size)}]")
        print(f"KEEP : {keep.path} [{keep.width}x{keep.height}, {human_mb(keep.size)}]\n")

    if args.dry_run:
        print("Dry run: no changes made.")
        return

    print("Processing duplicates...")
    for dup, _ in tqdm(total_actions, desc="Removing Duplicates", unit="img"):
        try:
            if args.delete:
                dup.path.unlink(missing_ok=True)
            elif qdir:
                rel = dup.path.relative_to(root) if dup.path.is_relative_to(root) else Path(dup.path.name)
                dest = qdir / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.move(str(dup.path), str(dest))
        except Exception as e:
            print(f"Failed: {dup.path} -> {e}")

    print("\nDone.")
    print(f"Duplicates processed: {len(total_actions)} | Space freed (approx): {human_mb(total_space)}")
    if not args.delete and qdir:
        print(f"Duplicates moved to: {qdir}")

if __name__ == "__main__":
    main()
