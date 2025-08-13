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

try:
    from tqdm import tqdm
except ImportError:
    tqdm = lambda x, **k: x  # fallback if tqdm not installed

AUDIO_EXTS = {".mp3", ".m4a", ".aac", ".flac", ".wav", ".ogg", ".opus", ".wma", ".alac", ".ape", ".tta", ".aiff", ".aif"}
LOSSLESS_CODECS = {"flac", "wav", "alac", "ape", "tta", "pcm_s16le", "pcm_s24le", "aiff"}

@dataclass(frozen=True)
class AudioMeta:
    path: Path
    codec: Optional[str]
    bitrate: Optional[int]
    duration: Optional[float]
    size: int
    mtime: float

@dataclass(frozen=True)
class FingerprintEntry:
    fp_hash: str
    duration: int
    meta: AudioMeta

def run_cmd(cmd: List[str]) -> Tuple[int, str, str]:
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = proc.communicate()
    return proc.returncode, out, err

def probe_ffprobe(path: Path) -> Tuple[Optional[str], Optional[int], Optional[float]]:
    cmd = [
        "ffprobe", "-v", "error", "-select_streams", "a:0",
        "-show_entries", "stream=codec_name,bit_rate:format=duration",
        "-of", "json", str(path)
    ]
    rc, out, _ = run_cmd(cmd)
    if rc != 0:
        return None, None, None
    try:
        data = json.loads(out)
        codec = None
        bitrate = None
        duration = None
        if "streams" in data and data["streams"]:
            s = data["streams"][0]
            codec = s.get("codec_name")
            if s.get("bit_rate"):
                try:
                    bitrate = int(s["bit_rate"])
                except ValueError:
                    bitrate = None
        if "format" in data and data["format"].get("duration"):
            try:
                duration = float(data["format"]["duration"])
            except ValueError:
                duration = None
        return codec, bitrate, duration
    except json.JSONDecodeError:
        return None, None, None

def fpcalc(path: Path) -> Optional[Tuple[str, int]]:
    cmd = ["fpcalc", "-json", str(path)]
    rc, out, err = run_cmd(cmd)
    if rc != 0:
        return None
    try:
        data = json.loads(out)
        fp = data.get("fingerprint")
        dur = int(round(float(data.get("duration", 0))))
        if not fp or dur <= 0:
            return None
        sha = hashlib.sha1(fp.encode("utf-8")).hexdigest()
        return sha, dur
    except Exception:
        return None

def is_audio_file(p: Path) -> bool:
    return p.suffix.lower() in AUDIO_EXTS

def scan_file(path: Path) -> Optional[FingerprintEntry]:
    try:
        fp = fpcalc(path)
        if not fp:
            return None
        fp_hash, dur = fp
        codec, bitrate, duration = probe_ffprobe(path)
        stat = path.stat()
        meta = AudioMeta(
            path=path,
            codec=codec,
            bitrate=bitrate,
            duration=duration if duration else float(dur),
            size=stat.st_size,
            mtime=stat.st_mtime
        )
        return FingerprintEntry(fp_hash=fp_hash, duration=dur, meta=meta)
    except Exception:
        return None

def best_candidate(candidates: List[AudioMeta]) -> AudioMeta:
    def is_lossless(codec: Optional[str]) -> int:
        return 1 if codec and codec.lower() in LOSSLESS_CODECS else 0

    def bitrate_or_estimate(meta: AudioMeta) -> int:
        if meta.bitrate and meta.bitrate > 0:
            return meta.bitrate
        if meta.duration and meta.duration > 0:
            return int((meta.size / meta.duration) * 8)
        return 0

    return sorted(
        candidates,
        key=lambda m: (
            -is_lossless(m.codec),
            -bitrate_or_estimate(m),
            -m.size,
            -m.mtime,
            len(str(m.path))
        ),
    )[0]

def walk_audio_files(root: Path) -> List[Path]:
    return [Path(dirpath) / name
            for dirpath, _, filenames in os.walk(root)
            for name in filenames if is_audio_file(Path(name))]

def main():
    parser = argparse.ArgumentParser(description="Deduplicate same-audio files using Chromaprint fingerprints.")
    parser.add_argument("music_dir", type=str, help="Music folder to scan")
    parser.add_argument("--threads", type=int, default=os.cpu_count() or 4)
    parser.add_argument("--dry-run", action="store_true", help="Preview actions only")
    parser.add_argument("--move-to", type=str, default=None, help="Folder to move duplicates to")
    parser.add_argument("--delete", action="store_true", help="Delete duplicates")
    parser.add_argument("--min-duration", type=int, default=20, help="Ignore files shorter than this (sec)")
    parser.add_argument("--same-duration-tolerance", type=int, default=2)
    args = parser.parse_args()

    if args.delete and args.move_to:
        print("Choose either --delete or --move-to, not both.")
        return

    root = Path(args.music_dir).expanduser().resolve()
    if not root.exists():
        print(f"Path not found: {root}")
        return

    qdir = None
    if args.move_to:
        qdir = Path(args.move_to).expanduser().resolve()
        qdir.mkdir(parents=True, exist_ok=True)

    files = walk_audio_files(root)
    print(f"Scanning {len(files)} files with {args.threads} threads...")
    entries: List[FingerprintEntry] = []
    with futures.ThreadPoolExecutor(max_workers=args.threads) as ex:
        for res in tqdm(ex.map(scan_file, files), total=len(files)):
            if res and res.duration >= args.min_duration:
                entries.append(res)

    groups: Dict[str, List[AudioMeta]] = {}
    for e in entries:
        key = f"{e.fp_hash}:{round(e.duration / max(args.same_duration_tolerance, 1))}"
        groups.setdefault(key, []).append(e.meta)

    dup_sets = [v for v in groups.values() if len(v) > 1]
    print(f"Found {len(dup_sets)} groups of duplicates.")

    actions = []
    for metas in dup_sets:
        keeper = best_candidate(metas)
        for m in metas:
            if m is not keeper:
                actions.append((m, keeper))

    for dup, keep in actions:
        print(f"DUP:  {dup.path}\nKEEP: {keep.path}\n")

    if args.dry_run:
        print("Dry run: no changes made.")
        return

    for dup, _ in actions:
        if args.delete:
            dup.path.unlink(missing_ok=True)
        elif qdir:
            rel = dup.path.relative_to(root)
            dest = qdir / rel
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(dup.path), str(dest))

if __name__ == "__main__":
    main()
