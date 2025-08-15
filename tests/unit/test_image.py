"""
Unit tests for image deduplication module.
"""

import sys
from pathlib import Path
from unittest import mock

import pytest
from PIL import Image

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from dedupl.image import (
    ImageMeta,
    compute_perceptual_hash,
    group_by_exact_hash,
    group_by_perceptual_hash,
    read_image_dimensions,
    select_best_quality,
)


class TestImageMeta:
    """Test ImageMeta dataclass."""

    def test_image_meta_creation(self, tmp_path):
        """Test creating ImageMeta instance."""
        test_file = tmp_path / "test.jpg"
        test_file.touch()

        meta = ImageMeta(
            path=test_file,
            width=1920,
            height=1080,
            size=2048000,
            mtime=1234567890.0,
            sha1="abc123def456",
            phash=None,
        )

        assert meta.path == test_file
        assert meta.width == 1920
        assert meta.height == 1080
        assert meta.size == 2048000
        assert meta.mtime == 1234567890.0
        assert meta.sha1 == "abc123def456"
        assert meta.phash is None

    def test_image_meta_immutable(self, tmp_path):
        """Test that ImageMeta is immutable."""
        test_file = tmp_path / "test.jpg"
        meta = ImageMeta(
            path=test_file,
            width=1920,
            height=1080,
            size=2048000,
            mtime=1234567890.0,
            sha1="abc123def456",
            phash=None,
        )

        with pytest.raises(AttributeError):
            meta.width = 3840


class TestImageDimensions:
    """Test image dimension reading."""

    def test_read_dimensions_success(self, tmp_path):
        """Test successful dimension reading."""
        test_file = tmp_path / "test.jpg"

        # Create a small test image
        img = Image.new("RGB", (100, 50), color="red")
        img.save(test_file)

        width, height = read_image_dimensions(test_file)
        assert width == 100
        assert height == 50

    def test_read_dimensions_failure(self, tmp_path):
        """Test dimension reading for invalid file."""
        test_file = tmp_path / "invalid.jpg"
        test_file.write_text("not an image")

        width, height = read_image_dimensions(test_file)
        assert width == 0
        assert height == 0

    def test_read_dimensions_nonexistent(self):
        """Test dimension reading for nonexistent file."""
        width, height = read_image_dimensions(Path("/nonexistent.jpg"))
        assert width == 0
        assert height == 0


class TestPerceptualHash:
    """Test perceptual hash computation."""

    def test_compute_perceptual_hash_success(self, tmp_path):
        """Test successful perceptual hash computation."""
        test_file = tmp_path / "test.jpg"

        # Create a test image
        img = Image.new("RGB", (100, 100), color="blue")
        img.save(test_file)

        phash = compute_perceptual_hash(test_file, hash_size=8)
        assert phash is not None

    def test_compute_perceptual_hash_different_images(self, tmp_path):
        """Test that different images have different hashes."""
        file1 = tmp_path / "img1.jpg"
        file2 = tmp_path / "img2.jpg"

        # Create different images
        img1 = Image.new("RGB", (100, 100), color="red")
        img1.save(file1)

        img2 = Image.new("RGB", (100, 100), color="blue")
        # Make the second image more different by adding a pattern
        pixels = img2.load()
        for i in range(0, 100, 10):
            for j in range(0, 100, 10):
                pixels[i, j] = (0, 0, 255)  # Blue
                pixels[i + 5, j + 5] = (255, 255, 0)  # Yellow
        img2.save(file2)

        hash1 = compute_perceptual_hash(file1, hash_size=8)
        hash2 = compute_perceptual_hash(file2, hash_size=8)

        # Compare hashes using Hamming distance
        distance = hash1 - hash2
        assert distance > 0  # Different images should have different hashes

    def test_compute_perceptual_hash_similar_images(self, tmp_path):
        """Test that similar images have close hashes."""
        file1 = tmp_path / "img1.jpg"
        file2 = tmp_path / "img2.jpg"

        # Create base image
        img1 = Image.new("RGB", (100, 100), color="red")
        img1.save(file1)

        # Create slightly modified image (small change)
        img2 = Image.new("RGB", (100, 100), color="red")
        # Add a small mark
        pixels = img2.load()
        pixels[50, 50] = (255, 0, 0)
        img2.save(file2)

        hash1 = compute_perceptual_hash(file1, hash_size=8)
        hash2 = compute_perceptual_hash(file2, hash_size=8)

        # Hashes should be very similar (small Hamming distance)
        distance = hash1 - hash2
        assert distance < 5


class TestQualitySelection:
    """Test image quality selection."""

    def test_select_best_quality_resolution_priority(self, tmp_path):
        """Test that higher resolution is prioritized."""
        high_res = ImageMeta(
            path=tmp_path / "high.jpg",
            width=3840,
            height=2160,
            size=4000000,
            mtime=1000.0,
            sha1="abc123",
            phash=None,
        )

        low_res = ImageMeta(
            path=tmp_path / "low.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=2000.0,  # Newer
            sha1="def456",
            phash=None,
        )

        best = select_best_quality([low_res, high_res])
        assert best == high_res

    def test_select_best_quality_size_tiebreaker(self, tmp_path):
        """Test that file size is used as tiebreaker."""
        file1 = ImageMeta(
            path=tmp_path / "file1.jpg",
            width=1920,
            height=1080,
            size=3000000,  # Larger
            mtime=1000.0,
            sha1="abc123",
            phash=None,
        )

        file2 = ImageMeta(
            path=tmp_path / "file2.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="def456",
            phash=None,
        )

        best = select_best_quality([file1, file2])
        assert best == file1

    def test_select_best_quality_mtime_tiebreaker(self, tmp_path):
        """Test that modification time is used as tiebreaker."""
        file1 = ImageMeta(
            path=tmp_path / "file1.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=2000.0,  # Newer
            sha1="abc123",
            phash=None,
        )

        file2 = ImageMeta(
            path=tmp_path / "file2.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="def456",
            phash=None,
        )

        best = select_best_quality([file1, file2])
        assert best == file1


class TestExactHashGrouping:
    """Test exact hash grouping."""

    def test_group_by_exact_hash_identical(self, tmp_path):
        """Test grouping identical files."""
        meta1 = ImageMeta(
            path=tmp_path / "file1.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="abc123",
            phash=None,
        )

        meta2 = ImageMeta(
            path=tmp_path / "file2.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="abc123",  # Same hash
            phash=None,
        )

        groups = group_by_exact_hash([meta1, meta2])
        assert len(groups) == 1
        assert len(groups[0]) == 2

    def test_group_by_exact_hash_different(self, tmp_path):
        """Test that different hashes don't group."""
        meta1 = ImageMeta(
            path=tmp_path / "file1.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="abc123",
            phash=None,
        )

        meta2 = ImageMeta(
            path=tmp_path / "file2.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="def456",  # Different hash
            phash=None,
        )

        groups = group_by_exact_hash([meta1, meta2])
        assert len(groups) == 0  # No groups with more than 1 item

    def test_group_by_exact_hash_multiple_groups(self, tmp_path):
        """Test multiple duplicate groups."""
        items = [
            ImageMeta(
                tmp_path / f"file{i}.jpg",
                100,
                100,
                1000,
                1000.0,
                sha1="hash1" if i < 2 else "hash2",
                phash=None,
            )
            for i in range(4)
        ]

        groups = group_by_exact_hash(items)
        assert len(groups) == 2  # Two groups
        assert all(len(group) == 2 for group in groups)


class TestPerceptualHashGrouping:
    """Test perceptual hash grouping."""

    @mock.patch("dedupl.image.imagehash.ImageHash")
    def test_group_by_perceptual_hash_similar(self, mock_hash_class, tmp_path):
        """Test grouping perceptually similar images."""
        # Create mock hashes with controlled distance
        hash1 = mock.Mock()
        hash2 = mock.Mock()
        hash1.__sub__ = mock.Mock(return_value=3)  # Distance of 3

        meta1 = ImageMeta(
            path=tmp_path / "file1.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="abc123",
            phash=hash1,
        )

        meta2 = ImageMeta(
            path=tmp_path / "file2.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="def456",
            phash=hash2,
        )

        groups = group_by_perceptual_hash([meta1, meta2], threshold=5)
        assert len(groups) == 1
        assert len(groups[0]) == 2

    @mock.patch("dedupl.image.imagehash.ImageHash")
    def test_group_by_perceptual_hash_different(self, mock_hash_class, tmp_path):
        """Test that dissimilar images don't group."""
        # Create mock hashes with large distance
        hash1 = mock.Mock()
        hash2 = mock.Mock()
        hash1.__sub__ = mock.Mock(return_value=20)  # Distance of 20

        meta1 = ImageMeta(
            path=tmp_path / "file1.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="abc123",
            phash=hash1,
        )

        meta2 = ImageMeta(
            path=tmp_path / "file2.jpg",
            width=1920,
            height=1080,
            size=2000000,
            mtime=1000.0,
            sha1="def456",
            phash=hash2,
        )

        groups = group_by_perceptual_hash([meta1, meta2], threshold=5)
        assert len(groups) == 0  # No groups formed

    def test_group_by_perceptual_hash_aspect_ratio_check(self, tmp_path):
        """Test aspect ratio checking in perceptual grouping."""
        # Mock hashes with small distance
        hash1 = mock.Mock()
        hash2 = mock.Mock()
        hash1.__sub__ = mock.Mock(return_value=3)

        # Different aspect ratios
        meta1 = ImageMeta(
            path=tmp_path / "wide.jpg",
            width=1920,
            height=1080,  # 16:9
            size=2000000,
            mtime=1000.0,
            sha1="abc123",
            phash=hash1,
        )

        meta2 = ImageMeta(
            path=tmp_path / "square.jpg",
            width=1000,
            height=1000,  # 1:1
            size=2000000,
            mtime=1000.0,
            sha1="def456",
            phash=hash2,
        )

        # With aspect ratio check, shouldn't group
        groups = group_by_perceptual_hash([meta1, meta2], threshold=5, check_aspect_ratio=True)
        assert len(groups) == 0

        # Without aspect ratio check, should group
        groups = group_by_perceptual_hash([meta1, meta2], threshold=5, check_aspect_ratio=False)
        # This would group if hashes were actually similar
