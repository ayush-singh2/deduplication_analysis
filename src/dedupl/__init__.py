"""
deDupl - Media file deduplication toolkit.

A Python toolkit for detecting and removing duplicate audio, image, and video files
using perceptual and content-based fingerprinting.
"""

__version__ = "0.2.0"
__author__ = "deDupl Contributors"

from .common import (
    DeduplicationConfig,
    DuplicateStats,
    check_external_dependency,
    format_file_size,
    setup_logging,
)

__all__ = [
    "DeduplicationConfig",
    "DuplicateStats",
    "check_external_dependency",
    "format_file_size",
    "setup_logging",
]
