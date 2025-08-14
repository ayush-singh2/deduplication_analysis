"""
Image file deduplication using perceptual hashing.
"""

import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from PIL import Image, ImageOps

try:
    import imagehash
except ImportError as err:
    raise ImportError("imagehash is required for image deduplication") from err

from .common import (
    file_sha1_hash,
    validate_path_security,
    walk_files_by_extension,
)

logger = logging.getLogger(__name__)

# Image file extensions to process
IMAGE_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".bmp", ".gif", ".tif", ".tiff", ".heic", ".heif"}


@dataclass(frozen=True)
class ImageMeta:
    """Metadata for an image file."""

    path: Path
    width: int
    height: int
    size: int
    mtime: float
    sha1: str
    phash: Optional[imagehash.ImageHash]


def compute_perceptual_hash(path: Path, hash_size: int = 16) -> Optional[imagehash.ImageHash]:
    """
    Compute perceptual hash of an image.

    Args:
        path: Path to image file
        hash_size: Size of the hash (default: 16)

    Returns:
        ImageHash object or None on error
    """
    if not validate_path_security(path):
        logger.warning(f"Skipping potentially unsafe path: {path}")
        return None

    try:
        with Image.open(path) as img:
            # Normalize orientation based on EXIF
            img = ImageOps.exif_transpose(img)
            # Convert to RGB to avoid mode issues
            img = img.convert("RGB")
            return imagehash.phash(img, hash_size=hash_size)
    except Exception as e:
        logger.debug(f"Failed to compute perceptual hash for {path}: {e}")
        return None


def read_image_dimensions(path: Path) -> Tuple[int, int]:
    """
    Read image dimensions.

    Args:
        path: Path to image file

    Returns:
        Tuple of (width, height) or (0, 0) on error
    """
    try:
        with Image.open(path) as img:
            img = ImageOps.exif_transpose(img)
            return img.width, img.height
    except Exception as e:
        logger.debug(f"Failed to read dimensions for {path}: {e}")
        return 0, 0


def scan_image_file(path: Path, hash_size: int = 16) -> Optional[ImageMeta]:
    """
    Scan an image file and extract metadata and hashes.

    Args:
        path: Path to image file
        hash_size: Size of perceptual hash

    Returns:
        ImageMeta object or None on error
    """
    try:
        # Calculate file hash
        sha1 = file_sha1_hash(path)
        if not sha1:
            return None

        # Read dimensions
        width, height = read_image_dimensions(path)

        # Compute perceptual hash
        phash = compute_perceptual_hash(path, hash_size)

        # Get file stats
        stat = path.stat()

        return ImageMeta(
            path=path,
            width=width,
            height=height,
            size=stat.st_size,
            mtime=stat.st_mtime,
            sha1=sha1,
            phash=phash,
        )
    except (OSError, PermissionError) as e:
        logger.debug(f"Cannot access file {path}: {e}")
        return None
    except Exception as e:
        logger.debug(f"Unexpected error scanning {path}: {e}")
        return None


def find_image_files(root: Path) -> List[Path]:
    """Find all image files in directory tree."""
    return walk_files_by_extension(root, IMAGE_EXTS)


def select_best_quality(candidates: List[ImageMeta]) -> ImageMeta:
    """
    Select the best quality image from duplicates.

    Prioritizes:
    1. Higher resolution (width * height)
    2. Larger file size
    3. Newer modification time
    4. Shorter path

    Args:
        candidates: List of duplicate images

    Returns:
        The best quality ImageMeta to keep
    """
    return sorted(
        candidates,
        key=lambda m: (-(m.width * m.height), -m.size, -m.mtime, len(str(m.path))),
    )[0]


def group_by_exact_hash(items: List[ImageMeta]) -> List[List[ImageMeta]]:
    """
    Group images by exact SHA-1 hash.

    Args:
        items: List of image metadata

    Returns:
        List of groups with exact duplicates
    """
    groups: Dict[str, List[ImageMeta]] = {}
    for item in items:
        groups.setdefault(item.sha1, []).append(item)
    return [group for group in groups.values() if len(group) > 1]


def group_by_perceptual_hash(
    items: List[ImageMeta], threshold: int = 6, check_aspect_ratio: bool = True
) -> List[List[ImageMeta]]:
    """
    Group images by perceptual hash similarity.

    Args:
        items: List of image metadata
        threshold: Maximum Hamming distance for similarity
        check_aspect_ratio: Whether to check aspect ratio similarity

    Returns:
        List of groups with perceptually similar images
    """
    # Filter to items with valid perceptual hash
    with_phash = [item for item in items if item.phash is not None]

    visited = set()
    groups = []

    # Sort by quality for consistent grouping
    sorted_items = sorted(with_phash, key=lambda m: (-(m.width * m.height), -m.size, -m.mtime))

    for i, base in enumerate(sorted_items):
        if base.path in visited:
            continue

        group = [base]
        visited.add(base.path)

        for candidate in sorted_items[i + 1 :]:
            if candidate.path in visited:
                continue

            # Calculate Hamming distance
            distance = base.phash - candidate.phash

            if distance <= threshold:
                # Optional aspect ratio check
                if check_aspect_ratio:
                    ar_base = base.width / base.height if base.height else 0
                    ar_cand = candidate.width / candidate.height if candidate.height else 0

                    # Allow 10% difference in aspect ratio
                    if ar_base and ar_cand:
                        ratio = ar_base / ar_cand
                        if not (0.9 <= ratio <= 1.11):
                            continue

                group.append(candidate)
                visited.add(candidate.path)

        if len(group) > 1:
            groups.append(group)

    return groups
