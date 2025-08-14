"""
Audio file deduplication using Chromaprint fingerprints.
"""

import hashlib
import json
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from .common import (
    execute_command,
    validate_path_security,
    walk_files_by_extension,
)

logger = logging.getLogger(__name__)

# Audio file extensions to process
AUDIO_EXTS = {
    ".mp3",
    ".m4a",
    ".aac",
    ".flac",
    ".wav",
    ".ogg",
    ".opus",
    ".wma",
    ".alac",
    ".ape",
    ".tta",
    ".aiff",
    ".aif",
}

# Lossless audio codecs for quality prioritization
LOSSLESS_CODECS = {"flac", "wav", "alac", "ape", "tta", "pcm_s16le", "pcm_s24le", "aiff"}


@dataclass(frozen=True)
class AudioMeta:
    """Metadata for an audio file."""

    path: Path
    codec: Optional[str]
    bitrate: Optional[int]
    duration: Optional[float]
    size: int
    mtime: float


@dataclass(frozen=True)
class FingerprintEntry:
    """Audio fingerprint entry with metadata."""

    fp_hash: str
    duration: int
    meta: AudioMeta


def probe_ffprobe(path: Path) -> Tuple[Optional[str], Optional[int], Optional[float]]:
    """
    Extract audio metadata using ffprobe.

    Args:
        path: Path to audio file

    Returns:
        Tuple of (codec_name, bitrate_bps, duration_seconds)
    """
    if not validate_path_security(path):
        logger.warning(f"Skipping potentially unsafe path: {path}")
        return None, None, None

    cmd = [
        "ffprobe",
        "-v",
        "error",
        "-select_streams",
        "a:0",
        "-show_entries",
        "stream=codec_name,bit_rate:format=duration",
        "-of",
        "json",
        str(path),
    ]

    rc, out, err = execute_command(cmd, timeout=30)
    if rc != 0:
        logger.debug(f"ffprobe failed for {path}: {err}")
        return None, None, None

    try:
        data = json.loads(out)
        codec = None
        bitrate = None
        duration = None

        if "streams" in data and data["streams"]:
            stream = data["streams"][0]
            codec = stream.get("codec_name")
            if stream.get("bit_rate"):
                try:
                    bitrate = int(stream["bit_rate"])
                except (ValueError, TypeError):
                    bitrate = None

        if "format" in data and data["format"].get("duration"):
            try:
                duration = float(data["format"]["duration"])
            except (ValueError, TypeError):
                duration = None

        return codec, bitrate, duration
    except json.JSONDecodeError as e:
        logger.debug(f"Failed to parse ffprobe output for {path}: {e}")
        return None, None, None


def generate_fingerprint(path: Path) -> Optional[Tuple[str, int]]:
    """
    Generate audio fingerprint using Chromaprint's fpcalc.

    Args:
        path: Path to audio file

    Returns:
        Tuple of (fingerprint_hash, duration_seconds) or None on error
    """
    if not validate_path_security(path):
        logger.warning(f"Skipping potentially unsafe path: {path}")
        return None

    cmd = ["fpcalc", "-json", str(path)]
    rc, out, err = execute_command(cmd, timeout=60)

    if rc != 0:
        logger.debug(f"fpcalc failed for {path}: {err}")
        return None

    try:
        data = json.loads(out)
        fingerprint = data.get("fingerprint")
        duration = int(round(float(data.get("duration", 0))))

        if not fingerprint or duration <= 0:
            logger.debug(f"Invalid fingerprint data for {path}")
            return None

        # Hash the fingerprint for efficient comparison
        fp_hash = hashlib.sha1(fingerprint.encode("utf-8")).hexdigest()
        return fp_hash, duration
    except (json.JSONDecodeError, ValueError, TypeError) as e:
        logger.debug(f"Failed to parse fpcalc output for {path}: {e}")
        return None


def scan_audio_file(path: Path) -> Optional[FingerprintEntry]:
    """
    Scan an audio file and extract fingerprint and metadata.

    Args:
        path: Path to audio file

    Returns:
        FingerprintEntry or None on error
    """
    try:
        # Generate fingerprint
        fingerprint_data = generate_fingerprint(path)
        if not fingerprint_data:
            return None

        fp_hash, duration_fp = fingerprint_data

        # Extract metadata
        codec, bitrate, duration_probe = probe_ffprobe(path)

        # Get file stats
        stat = path.stat()

        # Use probe duration if available, otherwise use fingerprint duration
        duration = duration_probe if duration_probe else float(duration_fp)

        meta = AudioMeta(
            path=path,
            codec=codec,
            bitrate=bitrate,
            duration=duration,
            size=stat.st_size,
            mtime=stat.st_mtime,
        )

        return FingerprintEntry(fp_hash=fp_hash, duration=duration_fp, meta=meta)
    except (OSError, PermissionError) as e:
        logger.debug(f"Cannot access file {path}: {e}")
        return None
    except Exception as e:
        logger.debug(f"Unexpected error scanning {path}: {e}")
        return None


def find_audio_files(root: Path) -> List[Path]:
    """Find all audio files in directory tree."""
    return walk_files_by_extension(root, AUDIO_EXTS)


def select_best_quality(candidates: List[AudioMeta]) -> AudioMeta:
    """
    Select the best quality audio file from duplicates.

    Prioritizes:
    1. Lossless codecs
    2. Higher bitrate
    3. Larger file size
    4. Newer modification time
    5. Shorter path (likely original location)

    Args:
        candidates: List of duplicate audio files

    Returns:
        The best quality AudioMeta to keep
    """

    def is_lossless(codec: Optional[str]) -> int:
        """Check if codec is lossless."""
        return 1 if codec and codec.lower() in LOSSLESS_CODECS else 0

    def bitrate_or_estimate(meta: AudioMeta) -> int:
        """Get bitrate or estimate from file size."""
        if meta.bitrate and meta.bitrate > 0:
            return meta.bitrate
        if meta.duration and meta.duration > 0:
            # Estimate bitrate from file size and duration
            return int((meta.size / meta.duration) * 8)
        return 0

    return sorted(
        candidates,
        key=lambda m: (
            -is_lossless(m.codec),
            -bitrate_or_estimate(m),
            -m.size,
            -m.mtime,
            len(str(m.path)),
        ),
    )[0]


def group_duplicates(
    entries: List[FingerprintEntry], duration_tolerance: int = 2
) -> Dict[str, List[AudioMeta]]:
    """
    Group audio files by fingerprint and duration.

    Args:
        entries: List of fingerprint entries
        duration_tolerance: Tolerance in seconds for duration matching

    Returns:
        Dictionary mapping group keys to lists of duplicate AudioMeta
    """
    groups: Dict[str, List[AudioMeta]] = {}

    for entry in entries:
        # Create key from fingerprint hash and rounded duration
        duration_bucket = round(entry.duration / max(duration_tolerance, 1))
        key = f"{entry.fp_hash}:{duration_bucket}"
        groups.setdefault(key, []).append(entry.meta)

    return groups
