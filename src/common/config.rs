//! Deduplication configuration — mirrors Python's `DeduplicationConfig`.

use std::path::{Path, PathBuf};

use crate::error::{DeduplError, Result};
use tracing::warn;

/// Common configuration for all deduplication types.
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    pub root_dir: PathBuf,
    pub threads: usize,
    pub dry_run: bool,
    pub move_to: Option<PathBuf>,
    pub delete: bool,
}

impl DeduplicationConfig {
    /// Create a new configuration.
    pub fn new(
        root_dir: PathBuf,
        threads: usize,
        dry_run: bool,
        move_to: Option<PathBuf>,
        delete: bool,
    ) -> Self {
        Self {
            root_dir,
            threads,
            dry_run,
            move_to,
            delete,
        }
    }

    /// Validate configuration settings.
    ///
    /// - `--delete` and `--move-to` are mutually exclusive.
    /// - `root_dir` must exist.
    /// - The quarantine directory is created if necessary.
    pub fn validate(&self) -> Result<()> {
        if self.delete && self.move_to.is_some() {
            return Err(DeduplError::ConflictingOptions);
        }

        if !self.root_dir.exists() {
            return Err(DeduplError::RootDirNotFound(self.root_dir.clone()));
        }

        if let Some(ref move_to) = self.move_to {
            // Warn if quarantine dir is inside the scan dir.
            if let (Ok(resolved_move), Ok(resolved_root)) =
                (move_to.canonicalize(), self.root_dir.canonicalize())
            {
                if resolved_move.starts_with(&resolved_root) {
                    warn!(
                        "Quarantine directory is within scan directory — this may cause issues"
                    );
                }
            }

            // Create quarantine directory if needed.
            std::fs::create_dir_all(move_to).map_err(|e| DeduplError::Io {
                path: move_to.clone(),
                source: e,
            })?;
        }

        Ok(())
    }
}

/// Check if an external command is available on `PATH`.
///
/// Uses `which` on Unix and `where` on Windows — mirrors the Python
/// `check_external_dependency` function.
pub fn check_external_dependency(command: &str) -> bool {
    let lookup = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    std::process::Command::new(lookup)
        .arg(command)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Set up the `tracing` subscriber.
///
/// `verbose = true` enables `DEBUG` level; otherwise `INFO`.
pub fn setup_logging(verbose: bool) {
    use tracing_subscriber::EnvFilter;

    let filter = if verbose { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .init();
}

/// Process duplicate files according to configuration.
///
/// Returns the number of successfully processed files.
pub fn process_duplicate_actions<P: AsRef<Path>>(
    actions: &[(P, P)],
    config: &DeduplicationConfig,
) -> anyhow::Result<usize> {
    if config.dry_run {
        tracing::info!("Dry run mode — no files will be modified");
        return Ok(0);
    }

    let mut processed: usize = 0;

    for (dup, _keeper) in actions {
        let dup_path = dup.as_ref();

        if config.delete {
            match std::fs::remove_file(dup_path) {
                Ok(()) => {
                    tracing::debug!("Deleted: {}", dup_path.display());
                    processed += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to delete {}: {e}", dup_path.display());
                }
            }
        } else if let Some(ref move_to) = config.move_to {
            let rel = dup_path
                .strip_prefix(&config.root_dir)
                .unwrap_or_else(|_| Path::new(dup_path.file_name().unwrap_or_default()));

            let mut dest = move_to.join(rel);

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| DeduplError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }

            // Avoid overwriting existing files.
            if dest.exists() {
                let stem = dest
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let ext = dest
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default();
                let parent_dir = dest
                    .parent()
                    .unwrap_or(move_to)
                    .to_path_buf();
                let mut counter = 1u32;
                loop {
                    dest = parent_dir.join(format!("{stem}_{counter}{ext}"));
                    if !dest.exists() {
                        break;
                    }
                    counter += 1;
                }
            }

            match std::fs::rename(dup_path, &dest) {
                Ok(()) => {
                    tracing::debug!("Moved: {} -> {}", dup_path.display(), dest.display());
                    processed += 1;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to move {} -> {}: {e}",
                        dup_path.display(),
                        dest.display()
                    );
                }
            }
        }
    }

    Ok(processed)
}
