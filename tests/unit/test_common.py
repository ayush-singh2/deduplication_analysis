"""
Unit tests for common utilities module.
"""

import os
import sys
from pathlib import Path
from unittest import mock

import pytest

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from dedupl.common import (
    DeduplicationConfig,
    DuplicateStats,
    check_external_dependency,
    execute_command,
    file_sha1_hash,
    format_file_size,
    validate_path_security,
    walk_files_by_extension,
)


class TestDeduplicationConfig:
    """Test DeduplicationConfig class."""

    def test_valid_config(self, tmp_path):
        """Test creating a valid configuration."""
        config = DeduplicationConfig(
            root_dir=tmp_path, threads=4, dry_run=True, move_to=None, delete=False
        )
        config.validate()
        assert config.root_dir == tmp_path
        assert config.threads == 4
        assert config.dry_run is True

    def test_invalid_both_delete_and_move(self, tmp_path):
        """Test that both delete and move_to raises error."""
        move_dir = tmp_path / "quarantine"
        config = DeduplicationConfig(
            root_dir=tmp_path, threads=4, dry_run=False, move_to=move_dir, delete=True
        )
        with pytest.raises(ValueError, match="Choose either"):
            config.validate()

    def test_nonexistent_root_dir(self):
        """Test that nonexistent root directory raises error."""
        config = DeduplicationConfig(
            root_dir=Path("/nonexistent/path"), threads=4, dry_run=True, move_to=None, delete=False
        )
        with pytest.raises(FileNotFoundError):
            config.validate()

    def test_creates_quarantine_dir(self, tmp_path):
        """Test that quarantine directory is created if needed."""
        quarantine = tmp_path / "quarantine"
        config = DeduplicationConfig(
            root_dir=tmp_path, threads=4, dry_run=False, move_to=quarantine, delete=False
        )
        config.validate()
        assert quarantine.exists()


class TestPathSecurity:
    """Test path security validation."""

    def test_valid_path(self, tmp_path):
        """Test that valid paths pass security check."""
        valid_file = tmp_path / "test.txt"
        valid_file.touch()
        assert validate_path_security(valid_file) is True

    def test_path_traversal_detection(self):
        """Test that path traversal attempts are detected."""
        unsafe_path = Path("../../../etc/passwd")
        assert validate_path_security(unsafe_path) is False

    def test_home_expansion_detection(self):
        """Test that home directory expansion is detected."""
        unsafe_path = Path("~/.ssh/id_rsa")
        assert validate_path_security(unsafe_path) is False

    @pytest.mark.skipif(os.name == "nt", reason="Symlinks require admin on Windows")
    def test_symlink_detection(self, tmp_path):
        """Test that symbolic links are detected."""
        target = tmp_path / "target.txt"
        target.touch()
        symlink = tmp_path / "link.txt"
        symlink.symlink_to(target)
        assert validate_path_security(symlink) is False


class TestFileOperations:
    """Test file operation utilities."""

    def test_file_sha1_hash(self, tmp_path):
        """Test SHA-1 hash calculation."""
        test_file = tmp_path / "test.txt"
        test_content = b"Hello, World!"
        test_file.write_bytes(test_content)

        # Expected SHA-1 hash of "Hello, World!"
        expected_hash = "0a0a9f2a6772942557ab5355d76af442f8f65e01"

        assert file_sha1_hash(test_file) == expected_hash

    def test_file_sha1_hash_nonexistent(self):
        """Test SHA-1 hash for nonexistent file."""
        result = file_sha1_hash(Path("/nonexistent/file.txt"))
        assert result is None

    def test_walk_files_by_extension(self, tmp_path):
        """Test walking directory for files with specific extensions."""
        # Create test files
        (tmp_path / "file1.txt").touch()
        (tmp_path / "file2.txt").touch()
        (tmp_path / "file3.md").touch()
        (tmp_path / "subdir").mkdir()
        (tmp_path / "subdir" / "file4.txt").touch()

        # Find .txt files
        txt_files = walk_files_by_extension(tmp_path, {".txt"})
        assert len(txt_files) == 3

        # Find .md files
        md_files = walk_files_by_extension(tmp_path, {".md"})
        assert len(md_files) == 1

    def test_format_file_size(self):
        """Test human-readable file size formatting."""
        assert format_file_size(0) == "0.00 B"
        assert format_file_size(1024) == "1.00 KB"
        assert format_file_size(1024 * 1024) == "1.00 MB"
        assert format_file_size(1024 * 1024 * 1024) == "1.00 GB"
        assert format_file_size(1536) == "1.50 KB"


class TestCommandExecution:
    """Test command execution utilities."""

    def test_execute_command_success(self):
        """Test successful command execution."""
        if os.name == "nt":
            cmd = ["cmd", "/c", "echo", "test"]
        else:
            cmd = ["echo", "test"]

        rc, stdout, stderr = execute_command(cmd)
        assert rc == 0
        assert "test" in stdout

    def test_execute_command_failure(self):
        """Test failed command execution."""
        cmd = ["nonexistent_command_12345"]
        rc, stdout, stderr = execute_command(cmd)
        assert rc != 0

    def test_execute_command_with_timeout(self):
        """Test command execution with timeout."""
        if os.name == "nt":
            # On Windows, use a command that will actually hang
            cmd = ["cmd", "/c", "ping", "-n", "10", "127.0.0.1"]
        else:
            cmd = ["sleep", "10"]

        rc, stdout, stderr = execute_command(cmd, timeout=1)
        assert rc == -1
        assert "timed out" in stderr

    def test_execute_command_unsafe_characters(self):
        """Test that unsafe characters in commands are rejected."""
        unsafe_cmd = ["echo", "test; rm -rf /"]
        rc, stdout, stderr = execute_command(unsafe_cmd)
        assert rc == -1
        assert "unsafe" in stderr.lower()


class TestDependencyChecking:
    """Test external dependency checking."""

    @mock.patch("subprocess.run")
    def test_check_dependency_exists(self, mock_run):
        """Test checking for existing dependency."""
        mock_run.return_value.returncode = 0
        assert check_external_dependency("python") is True

    @mock.patch("subprocess.run")
    def test_check_dependency_missing(self, mock_run):
        """Test checking for missing dependency."""
        mock_run.return_value.returncode = 1
        assert check_external_dependency("nonexistent_tool") is False


class TestDuplicateStats:
    """Test DuplicateStats class."""

    def test_stats_initialization(self):
        """Test stats initialization."""
        stats = DuplicateStats()
        assert stats.total_files == 0
        assert stats.duplicate_groups == 0
        assert stats.duplicate_files == 0
        assert stats.space_to_free == 0
        assert stats.space_freed == 0
        assert stats.files_processed == 0

    def test_stats_calculation(self):
        """Test stats calculation from duplicate groups."""
        stats = DuplicateStats()

        # Mock duplicate groups
        duplicate_groups = [
            [mock.Mock(size=1000), mock.Mock(size=1000)],
            [mock.Mock(size=2000), mock.Mock(size=2000), mock.Mock(size=2000)],
        ]

        stats.calculate_space(duplicate_groups)
        assert stats.duplicate_groups == 2
        assert stats.duplicate_files == 3  # 1 + 2 duplicates

    def test_stats_summary_output(self, capsys):
        """Test stats summary printing."""
        stats = DuplicateStats()
        stats.total_files = 100
        stats.duplicate_groups = 5
        stats.duplicate_files = 10
        stats.space_to_free = 1024 * 1024 * 50  # 50 MB
        stats.files_processed = 8
        stats.space_freed = 1024 * 1024 * 40  # 40 MB

        stats.print_summary()
        captured = capsys.readouterr()

        assert "Total files scanned: 100" in captured.out
        assert "Duplicate groups found: 5" in captured.out
        assert "Duplicate files: 10" in captured.out
        assert "50.00 MB" in captured.out
        assert "Files processed: 8" in captured.out
        assert "40.00 MB" in captured.out
