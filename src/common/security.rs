//! Path security validation — mirrors Python's `validate_path_security`.

use std::path::Path;

use tracing::warn;

/// Validate that a path is safe to use.
///
/// Rejects:
/// - Path traversal (`..`)
/// - Home-directory expansion (`~` prefix)
/// - Symbolic links
pub fn validate_path_security(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check for path traversal.
    if path_str.contains("..") {
        warn!("Suspicious path pattern detected (traversal): {}", path_str);
        return false;
    }

    // Check for home-directory expansion.
    if path_str.starts_with('~') {
        warn!(
            "Suspicious path pattern detected (home expansion): {}",
            path_str
        );
        return false;
    }

    // Check for symbolic links (only when the path exists on disk).
    if path.is_symlink() {
        warn!("Symlink detected: {}", path_str);
        return false;
    }

    true
}
