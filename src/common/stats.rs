//! Deduplication statistics and display helpers.

use std::fmt;

/// Track and report deduplication statistics.
#[derive(Debug, Default)]
pub struct DuplicateStats {
    pub total_files: usize,
    pub duplicate_groups: usize,
    pub duplicate_files: usize,
    pub space_to_free: u64,
    pub space_freed: u64,
    pub files_processed: usize,
}

impl DuplicateStats {
    /// Create a zeroed stats instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate statistics from duplicate groups.
    ///
    /// Each inner `Vec` is a group of duplicates; the first element is assumed
    /// to be the "keeper".
    pub fn calculate_space<T>(&mut self, duplicate_groups: &[Vec<T>]) {
        self.duplicate_groups = duplicate_groups.len();
        self.duplicate_files = duplicate_groups
            .iter()
            .map(|g| if g.len() > 1 { g.len() - 1 } else { 0 })
            .sum();
    }

    /// Print a human-readable summary to stdout.
    pub fn print_summary(&self) {
        println!();
        println!("{}", "=".repeat(50));
        println!("Deduplication Summary:");
        println!("  Total files scanned: {}", self.total_files);
        println!("  Duplicate groups found: {}", self.duplicate_groups);
        println!("  Duplicate files: {}", self.duplicate_files);
        println!(
            "  Potential space to free: {}",
            format_file_size(self.space_to_free)
        );
        if self.files_processed > 0 {
            println!("  Files processed: {}", self.files_processed);
            println!("  Space freed: {}", format_file_size(self.space_freed));
        }
        println!("{}", "=".repeat(50));
        println!();
    }
}

/// Format a byte count into a human-readable string.
///
/// Mirrors the Python `format_file_size` function exactly, using base-1024
/// units: B, KB, MB, GB, TB, PB.
pub fn format_file_size(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;

    for unit in &units[..units.len() - 1] {
        if size < 1024.0 {
            return format!("{size:.2} {unit}");
        }
        size /= 1024.0;
    }

    format!("{size:.2} {}", units.last().expect("units is non-empty"))
}

impl fmt::Display for DuplicateStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "files={}, groups={}, dupes={}, potential={}",
            self.total_files,
            self.duplicate_groups,
            self.duplicate_files,
            format_file_size(self.space_to_free)
        )
    }
}
