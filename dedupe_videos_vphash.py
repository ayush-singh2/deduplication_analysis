#!/usr/bin/env python3
import argparse
import concurrent.futures as futures
import hashlib
import json
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

# Progress
try:
    from tqdm import tqdm
except ImportError:
    tqdm = lambda x, **k: x  # fallback if tqdm not installed

# Video frame sampling
import cv2
from PIL import Image
import numpy as np
import imagehash

VIDEO_EXTS = {
    ".mp4", ".mkv", ".mov", ".avi", ".wmv", ".flv", ".webm", ".m4v", ".mpg", ".mpeg", ".ts"
}

@dataclass(frozen=True)
class VideoMeta:
    path: Path
    width: int
    height: int
    bitrate: Optional[int]
    duration: float
    size: int
    mtime: float
    sha1: str
    # perceptual signature
    majority_hash: Optional[int]   # 64-bit int (phash with hash_size=8)
    sparse_hashes: Optional[List[int]]  # up to N 64-bit ints, sampled over time

def is_video_file(p: Path) -> bool:
    return p.suffix.lower() in VIDEO_EXTS

def run_cmd(cmd: List[str]) -> Tuple[int, str, str]:
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = proc.communicate()
    return proc.returncode, out, err

def ffprobe_video(path: Path) -> Tuple[int, int, Optional[int], float]:
    """Return (width, height, bitrate_bps, duration_sec)."""
    cmd = [
        "ffprobe", "-v", "error",
        "-select_streams", "v:0",
        "-show_entries", "stream=width,height,bit_rate:format=duration",
        "-of", "json", str(path)
    ]
    rc, out, _ = run_cmd(cmd)
    if rc != 0:
        return 0, 0, None, 0.0
    try:
        data = json.loads(out)
        width = height = 0
        bitrate = None
        duration = 0.0
        if data.get("streams"):
            s = data["streams"][0]
            width = int(s.get("width") or 0)
            height = int(s.get("height") or 0)
            br = s.get("bit_rate")
            if br:
                try: bitrate = int(br)
                except: bitrate = None
        if data.get("format") and data["format"].get("duration"):
            try: duration = float(data["format"]["duration"])
            except: duration = 0.0
        return width, height, bitrate, duration
    except Exception:
        return 0, 0, None, 0.0

def file_sha1(path: Path, chunk: int = 1024 * 1024) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        while True:
            b = f.read(chunk)
            if not b: break
            h.update(b)
    return h.hexdigest()

def frame_phash_from_bgr(frame: np.ndarray, hash_size: int = 8) -> int:
    # Convert OpenCV BGR frame to PIL Image RGB and compute pHash
    img = Image.fromarray(cv2.cvtColor(frame, cv2.COLOR_BGR2RGB))
    ph = imagehash.phash(img, hash_size=hash_size)
    # Convert boolean array to 64-bit int
    bits = ph.hash.flatten()
    val = 0
    for bit in bits:
        val = (val << 1) | int(bool(bit))
    return int(val)

def sample_video_hashes(path: Path, fps_sample: float, max_secs: int, hash_size: int = 8) -> List[int]:
    """
    Sample pHash of frames at ~fps_sample up to max_secs.
    Returns list of 64-bit int hashes (length <= fps_sample*max_secs).
    """
    cap = cv2.VideoCapture(str(path))
    if not cap.isOpened():
        return []
    try:
        native_fps = cap.get(cv2.CAP_PROP_FPS) or 0.0
        total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT) or 0)
        duration = total_frames / native_fps if native_fps > 0 else None

        # Step between frames (in frames) to approximate fps_sample
        step = int(round((native_fps / fps_sample))) if native_fps and native_fps > 0 else None
        hashes: List[int] = []

        if step and step > 0:
            # Sample first max_secs seconds
            max_frames = int(min(total_frames, (duration if duration else max_secs) and (max_secs * native_fps)))
            f = 0
            while f < max_frames:
                cap.set(cv2.CAP_PROP_POS_FRAMES, f)
                ok, frame = cap.read()
                if not ok:
                    break
                try:
                    h = frame_phash_from_bgr(frame, hash_size=hash_size)
                    hashes.append(h)
                except Exception:
                    pass
                f += step
        else:
            # Fallback: read sequentially and sample every nth time
            nth = int(max(1, 30 / fps_sample))  # assume ~30fps as rough default
            count = 0
            limit_frames = int(fps_sample * max_secs)
            while len(hashes) < limit_frames:
                ok, frame = cap.read()
                if not ok:
                    break
                if (count % nth) == 0:
                    try:
                        h = frame_phash_from_bgr(frame, hash_size=hash_size)
                        hashes.append(h)
                    except Exception:
                        pass
                count += 1

        return hashes
    finally:
        cap.release()

def majority_hash(hashes: List[int]) -> Optional[int]:
    if not hashes:
        return None
    # Bitwise majority over 64-bit pHashes
    arr = np.array(hashes, dtype=np.uint64)
    bits = ((arr[:, None] >> np.arange(63, -1, -1)) & 1).astype(np.int32)  # shape (N,64)
    ones = bits.sum(axis=0)
    zeros = bits.shape[0] - ones
    maj_bits = (ones >= zeros).astype(np.uint8)
    val = 0
    for b in maj_bits:
        val = (val << 1) | int(b)
    return int(val)

def hamming(a: int, b: int) -> int:
    return int(bin((a ^ b) & ((1 << 64) - 1)).count("1"))

def best_quality(a: VideoMeta, b: VideoMeta) -> VideoMeta:
    # Prefer resolution (pixels) > bitrate > duration > size > mtime (newer)
    def px(m: VideoMeta): return m.width * m.height
    def br(m: VideoMeta): return m.bitrate or 0
    return sorted([a, b], key=lambda m: (-px(m), -br(m), -m.duration, -m.size, -m.mtime, len(str(m.path))))[0]

def compare_signatures(a: List[int], b: List[int], max_shift: int = 3) -> float:
    """
    Return minimal average Hamming distance allowing small temporal shift.
    If either is empty, return large number.
    """
    if not a or not b:
        return 1e9
    A = np.array(a, dtype=np.uint64)
    B = np.array(b, dtype=np.uint64)
    # Try shifts in [-max_shift, +max_shift]
    best = 1e9
    for shift in range(-max_shift, max_shift + 1):
        if shift >= 0:
            A_slice = A[shift: min(len(A), shift + len(B))]
            B_slice = B[:len(A_slice)]
        else:
            B_slice = B[-shift: min(len(B), -shift + len(A))]
            A_slice = A[:len(B_slice)]
        if len(A_slice) == 0 or len(B_slice) == 0:
            continue
        # Vectorized Hamming via bit counts
        x = np.bitwise_xor(A_slice, B_slice)
        # Python 3.11+: int.bit_count; for numpy uint64, use loop for portability
        dist = np.fromiter((int(int(v).bit_count()) for v in x.tolist()), dtype=np.int32, count=len(x))
        avg = float(dist.mean())
        if avg < best:
            best = avg
    return best

def scan_one(path: Path, fps_sample: float, max_secs: int, hash_size: int) -> Optional[VideoMeta]:
    try:
        sha = file_sha1(path)
        w, h, br, dur = ffprobe_video(path)
        st = path.stat()
        # Build signature quickly; cap time for speed
        sparse = sample_video_hashes(path, fps_sample=fps_sample, max_secs=max_secs, hash_size=hash_size)
        maj = majority_hash(sparse)
        return VideoMeta(
            path=path, width=w, height=h, bitrate=br, duration=dur,
            size=st.st_size, mtime=st.st_mtime, sha1=sha,
            majority_hash=maj, sparse_hashes=sparse
        )
    except Exception:
        return None

def walk_videos(root: Path) -> List[Path]:
    out = []
    for dp, _, fns in os.walk(root):
        for name in fns:
            p = Path(dp) / name
            if is_video_file(p):
                out.append(p)
    return out

def main():
    ap = argparse.ArgumentParser(description="Deduplicate videos via exact SHA-1 and perceptual pHash over sampled frames.")
    ap.add_argument("video_dir", help="Root folder to scan")
    ap.add_argument("--threads", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--dry-run", action="store_true", help="Preview only; do not move/delete")
    ap.add_argument("--move-to", type=str, default=None, help="Quarantine folder to move duplicates")
    ap.add_argument("--delete", action="store_true", help="Delete duplicates")
    ap.add_argument("--fps-sample", type=float, default=0.5, help="Frames per second to sample (e.g., 0.5 = 1 frame every 2s)")
    ap.add_argument("--max-secs", type=int, default=240, help="Max seconds to sample per video")
    ap.add_argument("--hash-size", type=int, default=8, help="pHash size (8 -> 64 bits)")
    ap.add_argument("--majority-threshold", type=int, default=8, help="Max Hamming (0-64) to consider majority-hash duplicate")
    ap.add_argument("--avg-threshold", type=float, default=10.0, help="Max average Hamming over sampled frames to consider duplicate")
    ap.add_argument("--duration-tol", type=float, default=3.0, help="Seconds tolerance to consider durations similar for grouping")
    args = ap.parse_args()

    if args.delete and args.move_to:
        print("Choose either --delete or --move-to, not both.")
        return

    root = Path(args.video_dir).expanduser().resolve()
    if not root.exists():
        print(f"Path not found: {root}")
        return

    qdir = None
    if args.move_to:
        qdir = Path(args.move_to).expanduser().resolve()
        qdir.mkdir(parents=True, exist_ok=True)

    files = walk_videos(root)
    print(f"Scanning {len(files)} video files with {args.threads} threads...")

    metas: List[VideoMeta] = []
    with futures.ThreadPoolExecutor(max_workers=args.threads) as ex:
        for m in tqdm(ex.map(lambda p: scan_one(p, args.fps_sample, args.max_secs, args.hash_size), files),
                      total=len(files), desc="Fingerprinting & Analyzing", unit="vid"):
            if m:
                metas.append(m)

    # Stage 1: exact file duplicates
    groups_by_sha: Dict[str, List[VideoMeta]] = {}
    for m in metas:
        groups_by_sha.setdefault(m.sha1, []).append(m)
    exact_actions = []
    exact_space = 0
    for group in groups_by_sha.values():
        if len(group) <= 1:
            continue
        keep = group[0]
        for g in group[1:]:
            keep = best_quality(keep, g)
        for g in group:
            if g is keep:
                continue
            exact_actions.append((g, keep))
            exact_space += g.size

    # Remove exact dupes from further processing
    exact_dupe_paths = {g.path for g, _ in exact_actions}
    remaining = [m for m in metas if m.path not in exact_dupe_paths]

    # Stage 2: perceptual duplicates
    # Bucket by (rounded duration) to avoid comparing very different videos
    buckets: Dict[int, List[VideoMeta]] = {}
    for m in remaining:
        key = int(round(m.duration / max(args.duration_tol, 0.1))) if m.duration > 0 else -1
        buckets.setdefault(key, []).append(m)

    phash_actions = []
    phash_space = 0

    for _, bucket in buckets.items():
        if len(bucket) < 2:
            continue
        # Greedy grouping:
        used = set()
        # Pre-collect majority hashes
        for i, a in enumerate(bucket):
            if a.path in used or a.majority_hash is None:
                continue
            group = [a]
            used.add(a.path)
            for j in range(i + 1, len(bucket)):
                b = bucket[j]
                if b.path in used or b.majority_hash is None:
                    continue
                # Quick check: majority hash distance
                d_major = hamming(a.majority_hash, b.majority_hash)
                if d_major > args.majority_threshold:
                    continue
                # Costly check: average distance with small temporal shift
                avg = compare_signatures(a.sparse_hashes or [], b.sparse_hashes or [], max_shift=3)
                if avg <= args.avg_threshold:
                    group.append(b)
                    used.add(b.path)
            if len(group) > 1:
                # Decide a single keeper for the group
                keeper = group[0]
                for g in group[1:]:
                    keeper = best_quality(keeper, g)
                for g in group:
                    if g is keeper:
                        continue
                    phash_actions.append((g, keeper))
                    phash_space += g.size

    total_actions = exact_actions + phash_actions
    total_space = exact_space + phash_space

    def human_mb(b: int) -> str:
        return f"{b/(1024*1024):.2f} MB"

    print(f"\nExact-duplicate groups: {len([v for v in groups_by_sha.values() if len(v) > 1])}")
    print(f"Perceptual-duplicate groups: ~{len(phash_actions)} files grouped (pairs inside groups).")
    print(f"Files to remove/move: {len(total_actions)} | Potential space to free: {human_mb(total_space)}\n")

    for dup, keep in total_actions:
        print(f"DUP  : {dup.path}  [{dup.width}x{dup.height}, ~{human_mb(dup.size)}, {dup.duration:.1f}s]")
        print(f"KEEP : {keep.path} [{keep.width}x{keep.height}, ~{human_mb(keep.size)}, {keep.duration:.1f}s]\n")

    if args.dry_run:
        print("Dry run: no changes made.")
        return

    print("Processing duplicates...")
    for dup, _ in tqdm(total_actions, desc="Removing Duplicates", unit="vid"):
        try:
            if args.delete:
                dup.path.unlink(missing_ok=True)
            elif qdir:
                try:
                    rel = dup.path.relative_to(root)
                except Exception:
                    rel = Path(dup.path.name)
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
