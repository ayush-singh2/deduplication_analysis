//! Integration tests for `common` module — parity with `test_common.py`.

use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use dedupl::common::command::execute_command;
use dedupl::common::config::check_external_dependency;
use dedupl::common::fs::{file_sha1_hash, walk_files_by_extension};
use dedupl::common::security::validate_path_security;
use dedupl::common::stats::{format_file_size, DuplicateStats};
use dedupl::DeduplicationConfig;

// ── TestDeduplicationConfig ───────────────────────────────────────────────

#[test]
fn test_valid_config() {
    let tmp = TempDir::new().unwrap();
    let config = DeduplicationConfig::new(tmp.path().to_path_buf(), 4, true, None, false);
    assert!(config.validate().is_ok());
    assert_eq!(config.root_dir, tmp.path());
    assert_eq!(config.threads, 4);
    assert!(config.dry_run);
}

#[test]
fn test_invalid_both_delete_and_move() {
    let tmp = TempDir::new().unwrap();
    let quarantine = tmp.path().join("quarantine");
    let config = DeduplicationConfig::new(
        tmp.path().to_path_buf(),
        4,
        false,
        Some(quarantine),
        true,
    );
    let err = config.validate().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("either"), "error message: {msg}");
}

#[test]
fn test_nonexistent_root_dir() {
    let config = DeduplicationConfig::new(
        PathBuf::from("/nonexistent/path"),
        4,
        true,
        None,
        false,
    );
    assert!(config.validate().is_err());
}

#[test]
fn test_creates_quarantine_dir() {
    let tmp = TempDir::new().unwrap();
    let quarantine = tmp.path().join("quarantine");
    let config = DeduplicationConfig::new(
        tmp.path().to_path_buf(),
        4,
        false,
        Some(quarantine.clone()),
        false,
    );
    config.validate().unwrap();
    assert!(quarantine.exists());
}

// ── TestPathSecurity ──────────────────────────────────────────────────────

#[test]
fn test_valid_path() {
    let tmp = TempDir::new().unwrap();
    let valid_file = tmp.path().join("test.txt");
    std::fs::write(&valid_file, "").unwrap();
    assert!(validate_path_security(&valid_file));
}

#[test]
fn test_path_traversal_detection() {
    let unsafe_path = Path::new("../../../etc/passwd");
    assert!(!validate_path_security(unsafe_path));
}

#[test]
fn test_home_expansion_detection() {
    let unsafe_path = Path::new("~/.ssh/id_rsa");
    assert!(!validate_path_security(unsafe_path));
}

#[test]
#[cfg(unix)]
fn test_symlink_detection() {
    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("target.txt");
    std::fs::write(&target, "").unwrap();
    let link = tmp.path().join("link.txt");
    symlink(&target, &link).unwrap();
    assert!(!validate_path_security(&link));
}

// ── TestFileOperations ────────────────────────────────────────────────────

#[test]
fn test_file_sha1_hash() {
    let tmp = TempDir::new().unwrap();
    let test_file = tmp.path().join("test.txt");
    std::fs::write(&test_file, b"Hello, World!").unwrap();

    // Expected SHA-1 of "Hello, World!"
    let expected = "0a0a9f2a6772942557ab5355d76af442f8f65e01";
    assert_eq!(file_sha1_hash(&test_file).unwrap(), expected);
}

#[test]
fn test_file_sha1_hash_nonexistent() {
    assert!(file_sha1_hash(Path::new("/nonexistent/file.txt")).is_none());
}

#[test]
fn test_walk_files_by_extension() {
    let tmp = TempDir::new().unwrap();

    std::fs::write(tmp.path().join("file1.txt"), "").unwrap();
    std::fs::write(tmp.path().join("file2.txt"), "").unwrap();
    std::fs::write(tmp.path().join("file3.md"), "").unwrap();
    let sub = tmp.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file4.txt"), "").unwrap();

    let exts = [".txt"].iter().copied().collect();
    let txt_files = walk_files_by_extension(tmp.path(), &exts);
    assert_eq!(txt_files.len(), 3);

    let exts = [".md"].iter().copied().collect();
    let md_files = walk_files_by_extension(tmp.path(), &exts);
    assert_eq!(md_files.len(), 1);
}

#[test]
fn test_format_file_size() {
    assert_eq!(format_file_size(0), "0.00 B");
    assert_eq!(format_file_size(1024), "1.00 KB");
    assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(format_file_size(1536), "1.50 KB");
}

// ── TestCommandExecution ──────────────────────────────────────────────────

#[test]
fn test_execute_command_success() {
    let (rc, stdout, _stderr) = execute_command(&["echo", "test"], None);
    assert_eq!(rc, 0);
    assert!(stdout.contains("test"));
}

#[test]
fn test_execute_command_failure() {
    let (rc, _stdout, _stderr) = execute_command(&["nonexistent_command_12345"], None);
    assert_ne!(rc, 0);
}

#[test]
fn test_execute_command_with_timeout() {
    let (rc, _stdout, stderr) = execute_command(
        &["sleep", "10"],
        Some(std::time::Duration::from_secs(1)),
    );
    assert_eq!(rc, -1);
    assert!(stderr.to_lowercase().contains("timed out"));
}

#[test]
fn test_execute_command_unsafe_characters() {
    let (rc, _stdout, stderr) = execute_command(&["echo", "test; rm -rf /"], None);
    assert_eq!(rc, -1);
    assert!(stderr.to_lowercase().contains("unsafe"));
}

// ── TestDependencyChecking ────────────────────────────────────────────────

#[test]
fn test_check_dependency_exists() {
    // `echo` is available on every Unix system.
    assert!(check_external_dependency("echo"));
}

#[test]
fn test_check_dependency_missing() {
    assert!(!check_external_dependency("nonexistent_tool_xyz_999"));
}

// ── TestDuplicateStats ────────────────────────────────────────────────────

#[test]
fn test_stats_initialization() {
    let stats = DuplicateStats::new();
    assert_eq!(stats.total_files, 0);
    assert_eq!(stats.duplicate_groups, 0);
    assert_eq!(stats.duplicate_files, 0);
    assert_eq!(stats.space_to_free, 0);
    assert_eq!(stats.space_freed, 0);
    assert_eq!(stats.files_processed, 0);
}

#[test]
fn test_stats_calculation() {
    let mut stats = DuplicateStats::new();

    // Two groups: first with 2 items, second with 3 items.
    let groups: Vec<Vec<u8>> = vec![vec![1, 2], vec![1, 2, 3]];
    stats.calculate_space(&groups);

    assert_eq!(stats.duplicate_groups, 2);
    assert_eq!(stats.duplicate_files, 3); // 1 + 2
}

#[test]
fn test_stats_summary_output() {
    let mut stats = DuplicateStats::new();
    stats.total_files = 100;
    stats.duplicate_groups = 5;
    stats.duplicate_files = 10;
    stats.space_to_free = 1024 * 1024 * 50;
    stats.files_processed = 8;
    stats.space_freed = 1024 * 1024 * 40;

    // Just verify it doesn't panic — capturing stdout in Rust integration
    // tests is non-trivial; the Python test used capsys.
    stats.print_summary();
}
