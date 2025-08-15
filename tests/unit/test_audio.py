"""
Unit tests for audio deduplication module.
"""

import json
import sys
from pathlib import Path
from unittest import mock

import pytest

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from dedupl.audio import (
    AudioMeta,
    FingerprintEntry,
    generate_fingerprint,
    group_duplicates,
    probe_ffprobe,
    select_best_quality,
)


class TestAudioMeta:
    """Test AudioMeta dataclass."""

    def test_audio_meta_creation(self, tmp_path):
        """Test creating AudioMeta instance."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        meta = AudioMeta(
            path=test_file,
            codec="mp3",
            bitrate=320000,
            duration=180.5,
            size=7200000,
            mtime=1234567890.0,
        )

        assert meta.path == test_file
        assert meta.codec == "mp3"
        assert meta.bitrate == 320000
        assert meta.duration == 180.5
        assert meta.size == 7200000
        assert meta.mtime == 1234567890.0

    def test_audio_meta_immutable(self, tmp_path):
        """Test that AudioMeta is immutable."""
        test_file = tmp_path / "test.mp3"
        meta = AudioMeta(
            path=test_file,
            codec="mp3",
            bitrate=320000,
            duration=180.5,
            size=7200000,
            mtime=1234567890.0,
        )

        with pytest.raises(AttributeError):
            meta.bitrate = 128000


class TestFingerprintGeneration:
    """Test audio fingerprint generation."""

    @mock.patch("dedupl.audio.execute_command")
    def test_generate_fingerprint_success(self, mock_execute, tmp_path):
        """Test successful fingerprint generation."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        # Mock fpcalc output
        mock_output = json.dumps({"fingerprint": "AQAADNFLFKJSDFLKJSDF", "duration": 180.5})
        mock_execute.return_value = (0, mock_output, "")

        result = generate_fingerprint(test_file)
        assert result is not None
        fp_hash, duration = result
        assert isinstance(fp_hash, str)
        assert len(fp_hash) == 40  # SHA-1 hash length
        assert duration == 180  # Rounded duration (180.5 rounds to 180)

    @mock.patch("dedupl.audio.execute_command")
    def test_generate_fingerprint_failure(self, mock_execute, tmp_path):
        """Test fingerprint generation failure."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        # Mock fpcalc failure
        mock_execute.return_value = (1, "", "Error: Failed to decode")

        result = generate_fingerprint(test_file)
        assert result is None

    @mock.patch("dedupl.audio.execute_command")
    def test_generate_fingerprint_invalid_output(self, mock_execute, tmp_path):
        """Test fingerprint generation with invalid output."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        # Mock invalid JSON output
        mock_execute.return_value = (0, "invalid json", "")

        result = generate_fingerprint(test_file)
        assert result is None


class TestFFprobeMetadata:
    """Test ffprobe metadata extraction."""

    @mock.patch("dedupl.audio.execute_command")
    def test_probe_ffprobe_success(self, mock_execute, tmp_path):
        """Test successful metadata extraction."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        # Mock ffprobe output
        mock_output = json.dumps(
            {
                "streams": [{"codec_name": "mp3", "bit_rate": "320000"}],
                "format": {"duration": "180.500000"},
            }
        )
        mock_execute.return_value = (0, mock_output, "")

        codec, bitrate, duration = probe_ffprobe(test_file)
        assert codec == "mp3"
        assert bitrate == 320000
        assert duration == 180.5

    @mock.patch("dedupl.audio.execute_command")
    def test_probe_ffprobe_missing_fields(self, mock_execute, tmp_path):
        """Test metadata extraction with missing fields."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        # Mock ffprobe output with missing fields
        mock_output = json.dumps(
            {
                "streams": [
                    {
                        "codec_name": "mp3"
                        # bit_rate missing
                    }
                ],
                "format": {},  # duration missing
            }
        )
        mock_execute.return_value = (0, mock_output, "")

        codec, bitrate, duration = probe_ffprobe(test_file)
        assert codec == "mp3"
        assert bitrate is None
        assert duration is None

    @mock.patch("dedupl.audio.execute_command")
    def test_probe_ffprobe_failure(self, mock_execute, tmp_path):
        """Test metadata extraction failure."""
        test_file = tmp_path / "test.mp3"
        test_file.touch()

        # Mock ffprobe failure
        mock_execute.return_value = (1, "", "Error: Invalid file")

        codec, bitrate, duration = probe_ffprobe(test_file)
        assert codec is None
        assert bitrate is None
        assert duration is None


class TestQualitySelection:
    """Test audio quality selection."""

    def test_select_best_quality_lossless_priority(self, tmp_path):
        """Test that lossless codecs are prioritized."""
        flac_file = AudioMeta(
            path=tmp_path / "test.flac",
            codec="flac",
            bitrate=None,
            duration=180.0,
            size=18000000,
            mtime=1000.0,
        )

        mp3_file = AudioMeta(
            path=tmp_path / "test.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=2000.0,  # Newer
        )

        best = select_best_quality([mp3_file, flac_file])
        assert best == flac_file

    def test_select_best_quality_bitrate_priority(self, tmp_path):
        """Test that higher bitrate is prioritized."""
        high_bitrate = AudioMeta(
            path=tmp_path / "high.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=1000.0,
        )

        low_bitrate = AudioMeta(
            path=tmp_path / "low.mp3",
            codec="mp3",
            bitrate=128000,
            duration=180.0,
            size=2880000,
            mtime=2000.0,  # Newer
        )

        best = select_best_quality([low_bitrate, high_bitrate])
        assert best == high_bitrate

    def test_select_best_quality_estimated_bitrate(self, tmp_path):
        """Test bitrate estimation from file size."""
        file1 = AudioMeta(
            path=tmp_path / "file1.mp3",
            codec="mp3",
            bitrate=None,  # No bitrate info
            duration=100.0,
            size=2000000,  # ~160kbps
            mtime=1000.0,
        )

        file2 = AudioMeta(
            path=tmp_path / "file2.mp3",
            codec="mp3",
            bitrate=None,  # No bitrate info
            duration=100.0,
            size=4000000,  # ~320kbps
            mtime=1000.0,
        )

        best = select_best_quality([file1, file2])
        assert best == file2


class TestDuplicateGrouping:
    """Test duplicate grouping logic."""

    def test_group_duplicates_exact_match(self, tmp_path):
        """Test grouping exact fingerprint matches."""
        meta1 = AudioMeta(
            path=tmp_path / "file1.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=1000.0,
        )

        meta2 = AudioMeta(
            path=tmp_path / "file2.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=1000.0,
        )

        entry1 = FingerprintEntry(fp_hash="abc123", duration=180, meta=meta1)

        entry2 = FingerprintEntry(fp_hash="abc123", duration=180, meta=meta2)

        groups = group_duplicates([entry1, entry2])
        assert len(groups) == 1
        assert len(list(groups.values())[0]) == 2

    def test_group_duplicates_duration_tolerance(self, tmp_path):
        """Test grouping with duration tolerance."""
        meta1 = AudioMeta(
            path=tmp_path / "file1.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=1000.0,
        )

        meta2 = AudioMeta(
            path=tmp_path / "file2.mp3",
            codec="mp3",
            bitrate=320000,
            duration=181.0,
            size=7240000,
            mtime=1000.0,
        )

        entry1 = FingerprintEntry(fp_hash="abc123", duration=180, meta=meta1)

        entry2 = FingerprintEntry(fp_hash="abc123", duration=181, meta=meta2)

        # With tolerance of 2 seconds, should group together
        groups = group_duplicates([entry1, entry2], duration_tolerance=2)
        assert len(groups) == 1
        assert len(list(groups.values())[0]) == 2

        # With tolerance of 0, should not group
        groups = group_duplicates([entry1, entry2], duration_tolerance=0)
        assert len(groups) == 2

    def test_group_duplicates_different_fingerprints(self, tmp_path):
        """Test that different fingerprints don't group."""
        meta1 = AudioMeta(
            path=tmp_path / "file1.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=1000.0,
        )

        meta2 = AudioMeta(
            path=tmp_path / "file2.mp3",
            codec="mp3",
            bitrate=320000,
            duration=180.0,
            size=7200000,
            mtime=1000.0,
        )

        entry1 = FingerprintEntry(fp_hash="abc123", duration=180, meta=meta1)

        entry2 = FingerprintEntry(fp_hash="def456", duration=180, meta=meta2)  # Different hash

        groups = group_duplicates([entry1, entry2])
        assert len(groups) == 2  # Two separate groups
