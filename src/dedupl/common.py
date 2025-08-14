#!/usr/bin/env python3
"""
Common utilities for media deduplication scripts.
Provides shared functionality for audio, image, and video deduplication.
"""

import argparse
import hashlib
import logging
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional, Set, Tuple

# Setup logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Try to import tqdm for progress bars
try:
    from tqdm import tqdm
except ImportError:
    # Fallback if tqdm not installed
    def tqdm(x, **k):
        return x


@dataclass
class DeduplicationConfig:
    """Common configuration for all deduplication types."""

    root_dir: Path
    threads: int
    dry_run: bool
    move_to: Optional[Path]
    delete: bool

    def validate(self) -> None:
        """Validate configuration settings."""
        if self.delete and self.move_to:
            raise ValueError("Choose either --delete or --move-to, not both.")

        if not self.root_dir.exists():
            raise FileNotFoundError(f"Path not found: {self.root_dir}")

        if self.move_to:
            # Security check: ensure quarantine dir is not within root dir
            try:
                if self.move_to.resolve().is_relative_to(self.root_dir.resolve()):
                    logger.warning(
                        "Quarantine directory is within scan directory - this may cause issues"
                    )
            except ValueError:
                pass  # Paths are not related, which is fine

            # Create quarantine directory if needed
            self.move_to.mkdir(parents=True, exist_ok=True)


def validate_path_security(path: Path) -> bool:
    """
    Validate that a path is safe to use.
    Checks for symlinks and path traversal attempts.
    """
    try:
        # Check if path contains suspicious patterns
        path_str = str(path)
        if ".." in path_str or path_str.startswith("~"):
            logger.warning(f"Suspicious path pattern detected: {path}")
            return False

        # Check for symlinks
        if path.is_symlink():
            logger.warning(f"Symlink detected: {path}")
            return False

        return True
    except Exception as e:
        logger.error(f"Error validating path {path}: {e}")
        return False


def walk_files_by_extension(root: Path, extensions: Set[str]) -> List[Path]:
    """
    Walk directory tree and return files matching given extensions.

    Args:
        root: Root directory to scan
        extensions: Set of file extensions to match (lowercase, with dots)

    Returns:
        List of Path objects for matching files
    """
    files = []
    try:
        for dirpath, _, filenames in os.walk(root):
            for name in filenames:
                path = Path(dirpath) / name
                if path.suffix.lower() in extensions and validate_path_security(path):
                    files.append(path)
    except Exception as e:
        logger.error(f"Error walking directory {root}: {e}")

    return files


def file_sha1_hash(path: Path, chunk_size: int = 1024 * 1024) -> Optional[str]:
    """
    Calculate SHA-1 hash of a file.

    Args:
        path: Path to file
        chunk_size: Size of chunks to read (default 1MB)

    Returns:
        Hexadecimal SHA-1 hash string, or None on error
    """
    try:
        h = hashlib.sha1()
        with path.open("rb") as f:
            while True:
                chunk = f.read(chunk_size)
                if not chunk:
                    break
                h.update(chunk)
        return h.hexdigest()
    except Exception as e:
        logger.error(f"Error hashing file {path}: {e}")
        return None


def execute_command(cmd: List[str], timeout: Optional[int] = None) -> Tuple[int, str, str]:
    """
    Execute external command safely with proper error handling.

    Args:
        cmd: Command and arguments as list
        timeout: Optional timeout in seconds

    Returns:
        Tuple of (return_code, stdout, stderr)
    """
    try:
        # Validate command doesn't contain shell metacharacters
        for arg in cmd:
            if any(c in arg for c in [";", "&", "|", ">", "<", "$", "`"]):
                logger.warning(f"Potentially unsafe character in command argument: {arg}")
                return (-1, "", "Command contains potentially unsafe characters")

        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            shell=False,  # Never use shell=True for security
        )
        stdout, stderr = proc.communicate(timeout=timeout)
        return proc.returncode, stdout, stderr
    except subprocess.TimeoutExpired:
        proc.kill()
        logger.error(f"Command timed out: {' '.join(cmd)}")
        return (-1, "", "Command timed out")
    except Exception as e:
        logger.error(f"Error executing command {cmd[0]}: {e}")
        return (-1, "", str(e))


def format_file_size(bytes_size: int) -> str:
    """
    Format file size in human-readable format.

    Args:
        bytes_size: Size in bytes

    Returns:
        Human-readable size string
    """
    for unit in ["B", "KB", "MB", "GB", "TB"]:
        if bytes_size < 1024.0:
            return f"{bytes_size:.2f} {unit}"
        bytes_size /= 1024.0
    return f"{bytes_size:.2f} PB"


def check_external_dependency(command: str) -> bool:
    """
    Check if an external command is available.

    Args:
        command: Command name to check

    Returns:
        True if command is available, False otherwise
    """
    try:
        result = subprocess.run(
            ["which", command] if os.name != "nt" else ["where", command],
            capture_output=True,
            text=True,
            check=False,
        )
        return result.returncode == 0
    except Exception:
        return False


def process_duplicate_actions(
    actions: List[Tuple], config: DeduplicationConfig, desc: str = "Removing Duplicates"
) -> int:
    """
    Process duplicate files according to configuration.

    Args:
        actions: List of (duplicate, keeper) tuples
        config: Deduplication configuration
        desc: Description for progress bar

    Returns:
        Number of successfully processed files
    """
    if config.dry_run:
        logger.info("Dry run mode - no files will be modified")
        return 0

    processed = 0
    failed = []

    for dup, _ in tqdm(actions, desc=desc, unit="file"):
        try:
            dup_path = dup.path if hasattr(dup, "path") else dup

            if config.delete:
                dup_path.unlink(missing_ok=True)
                logger.debug(f"Deleted: {dup_path}")
                processed += 1
            elif config.move_to:
                try:
                    rel = dup_path.relative_to(config.root_dir)
                except ValueError:
                    # Path is not relative to root, use just the filename
                    rel = Path(dup_path.name)

                dest = config.move_to / rel
                dest.parent.mkdir(parents=True, exist_ok=True)

                # Handle existing destination
                if dest.exists():
                    # Add suffix to avoid overwriting
                    stem = dest.stem
                    suffix = dest.suffix
                    counter = 1
                    while dest.exists():
                        dest = dest.parent / f"{stem}_{counter}{suffix}"
                        counter += 1

                shutil.move(str(dup_path), str(dest))
                logger.debug(f"Moved: {dup_path} -> {dest}")
                processed += 1
        except Exception as e:
            logger.error(f"Failed to process {dup_path}: {e}")
            failed.append((dup_path, str(e)))

    if failed:
        logger.warning(f"Failed to process {len(failed)} files")
        for path, error in failed[:5]:  # Show first 5 failures
            logger.warning(f"  {path}: {error}")

    return processed


def create_base_argument_parser(description: str) -> argparse.ArgumentParser:
    """
    Create base argument parser with common options.

    Args:
        description: Description for the parser

    Returns:
        ArgumentParser with common arguments
    """
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("scan_dir", type=str, help="Directory to scan for duplicates")
    parser.add_argument(
        "--threads",
        type=int,
        default=os.cpu_count() or 4,
        help="Number of threads to use (default: CPU count)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview actions only (no files are moved or deleted)",
    )
    parser.add_argument(
        "--move-to", type=str, default=None, help="Directory to move duplicate files to"
    )
    parser.add_argument(
        "--delete", action="store_true", help="Delete duplicate files (use with caution)"
    )
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose logging")
    return parser


def setup_logging(verbose: bool = False) -> None:
    """
    Setup logging configuration.

    Args:
        verbose: Enable verbose (DEBUG) logging
    """
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s", force=True
    )


class DuplicateStats:
    """Track and report deduplication statistics."""

    def __init__(self):
        self.total_files = 0
        self.duplicate_groups = 0
        self.duplicate_files = 0
        self.space_to_free = 0
        self.space_freed = 0
        self.files_processed = 0

    def calculate_space(self, duplicate_groups: List[List]) -> None:
        """Calculate potential space to free from duplicate groups."""
        self.duplicate_groups = len(duplicate_groups)
        self.duplicate_files = sum(len(group) - 1 for group in duplicate_groups)
        # Space calculation depends on specific implementation

    def print_summary(self) -> None:
        """Print summary statistics."""
        print(f"\n{'='*50}")
        print("Deduplication Summary:")
        print(f"  Total files scanned: {self.total_files}")
        print(f"  Duplicate groups found: {self.duplicate_groups}")
        print(f"  Duplicate files: {self.duplicate_files}")
        print(f"  Potential space to free: {format_file_size(self.space_to_free)}")
        if self.files_processed > 0:
            print(f"  Files processed: {self.files_processed}")
            print(f"  Space freed: {format_file_size(self.space_freed)}")
        print(f"{'='*50}\n")
