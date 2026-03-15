//! Library-level error definitions using `thiserror`.
//!
//! Application-level code uses `anyhow::Result` for propagation;
//! library-level functions expose these typed errors so callers can
//! match on specific failure modes.

use std::path::PathBuf;

/// Errors that can occur during deduplication operations.
#[derive(Debug, thiserror::Error)]
pub enum DeduplError {
    // ── Configuration ──────────────────────────────────────────────
    #[error("conflicting options: choose either --delete or --move-to, not both")]
    ConflictingOptions,

    #[error("root directory does not exist: {0}")]
    RootDirNotFound(PathBuf),

    // ── Security ───────────────────────────────────────────────────
    #[error("path traversal detected: {0}")]
    PathTraversal(PathBuf),

    #[error("home directory expansion detected: {0}")]
    HomeExpansion(PathBuf),

    #[error("symlink detected: {0}")]
    SymlinkDetected(PathBuf),

    #[error("unsafe shell metacharacter in command argument: {arg}")]
    UnsafeCommand { arg: String },

    // ── I/O ────────────────────────────────────────────────────────
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to hash file {0}")]
    HashError(PathBuf),

    // ── External tools ─────────────────────────────────────────────
    #[error("command `{cmd}` failed (exit code {code}): {stderr}")]
    CommandFailed {
        cmd: String,
        code: i32,
        stderr: String,
    },

    #[error("command `{cmd}` timed out")]
    CommandTimeout { cmd: String },

    #[error("external dependency `{0}` not found in PATH")]
    DependencyMissing(String),

    // ── Parsing ────────────────────────────────────────────────────
    #[error("JSON parse error for {context}: {source}")]
    JsonParse {
        context: String,
        source: serde_json::Error,
    },

    #[error("invalid data from {tool}: {detail}")]
    InvalidToolOutput { tool: String, detail: String },

    // ── Image-specific ─────────────────────────────────────────────
    #[error("image decode error for {path}: {detail}")]
    ImageDecode { path: PathBuf, detail: String },
}

/// Convenience alias used throughout the library.
pub type Result<T> = std::result::Result<T, DeduplError>;
