//! CLI entry point for `dedupl` — provides `audio` and `image` subcommands.

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use dedupl::audio::{self, AudioHasher};
use dedupl::common::config::{check_external_dependency, setup_logging};
use dedupl::common::{DuplicateStats, Hasher};
use dedupl::image::{self, ImageHasher};
use dedupl::DeduplicationConfig;

/// High-performance media file deduplication toolkit.
#[derive(Parser, Debug)]
#[command(name = "dedupl", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Deduplicate audio files using Chromaprint fingerprints.
    Audio(ScanArgs),
    /// Deduplicate image files using perceptual hashing.
    Image(ScanArgs),
    /// Deduplicate video files using frame fingerprinting.
    Video(ScanArgs),
    /// Deduplicate document files using content hashing.
    Document(ScanArgs),
}

/// Common arguments shared by all subcommands.
#[derive(Parser, Debug)]
struct ScanArgs {
    /// Directory to scan for duplicates.
    scan_dir: PathBuf,

    /// Number of threads to use (default: CPU count).
    #[arg(long, default_value_t = num_threads_default())]
    threads: usize,

    /// Preview actions only — no files are moved or deleted.
    #[arg(long)]
    dry_run: bool,

    /// Directory to move duplicate files to.
    #[arg(long)]
    move_to: Option<PathBuf>,

    /// Delete duplicate files (use with caution).
    #[arg(long)]
    delete: bool,

    /// Enable verbose (DEBUG) logging.
    #[arg(short, long)]
    verbose: bool,
}

fn num_threads_default() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audio(args) => run_audio(args),
        Commands::Image(args) => run_image(args),
        Commands::Video(args) => run_video(args),
        Commands::Document(args) => run_document(args),
    }
}
fn run_document(args: ScanArgs) -> anyhow::Result<()> {
    use dedupl::document::grouping::{group_by_sha1, group_by_ngram};
    use std::fs;
    use std::ffi::OsStr;
    setup_logging(args.verbose);

    let config = DeduplicationConfig::new(
        args.scan_dir.clone(),
        args.threads,
        args.dry_run,
        args.move_to,
        args.delete,
    );
    config.validate().context("Invalid configuration")?;

    // Collect document files (txt, pdf, docx)
    let doc_exts = ["txt", "pdf", "docx"];
    let files: Vec<_> = fs::read_dir(&config.root_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(OsStr::to_str).map(|ext| doc_exts.contains(&ext)).unwrap_or(false)
        })
        .collect();

    println!("Found {} document files", files.len());
    let file_strs: Vec<_> = files.iter().map(|p| p.to_str().unwrap()).collect();

    // Group by SHA-1 hash (exact duplicates)
    let groups = group_by_sha1(&file_strs)?;
    let dup_groups: Vec<Vec<_>> = groups.values().filter(|g| g.len() > 1).cloned().collect();

    println!("Found {} duplicate groups (exact hash)", dup_groups.len());
    for (i, group) in dup_groups.iter().enumerate() {
        println!("Group {}:", i + 1);
        for file in group {
            println!("  {}", file);
        }
    }

    Ok(())
}
fn run_video(args: ScanArgs) -> anyhow::Result<()> {
    use dedupl::video::{find_video_files, meta::VideoMeta, fingerprint::fingerprint_video, grouping::group_video_duplicates};
    use indicatif::ProgressBar;
    use std::path::Path;
    setup_logging(args.verbose);

    // Check external dependency
    if !check_external_dependency("ffprobe") {
        anyhow::bail!("ffprobe (FFmpeg) is not installed or not in PATH");
    }

    let config = DeduplicationConfig::new(
        args.scan_dir.clone(),
        args.threads,
        args.dry_run,
        args.move_to,
        args.delete,
    );
    config.validate().context("Invalid configuration")?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()
        .ok();

    let files = find_video_files(&config.root_dir);
    let pb = ProgressBar::new(files.len() as u64);

    // Extract metadata and fingerprints in parallel
    let results: Vec<_> = files.par_iter().map(|f| {
        let meta = VideoMeta::from_path(f);
        let fp = fingerprint_video(f, 30); // sample every 30 frames
        pb.inc(1);
        (meta, fp)
    }).collect();
    pb.finish_with_message("Scan complete");

    let metas: Vec<_> = results.iter().filter_map(|(m, _)| m.clone()).collect();
    let fps: Vec<_> = results.iter().filter_map(|(_, f)| f.clone()).collect();

    let groups = group_video_duplicates(&metas, &fps, 0.5); // 50% frame match threshold

    let mut stats = DuplicateStats::new();
    stats.total_files = files.len();
    let dup_groups: Vec<Vec<_>> = groups.values().filter(|g| g.len() > 1).cloned().collect();
    stats.calculate_space(&dup_groups);
    stats.print_summary();

    Ok(())
}

fn run_audio(args: ScanArgs) -> anyhow::Result<()> {
    setup_logging(args.verbose);

    // Check external dependencies.
    if !check_external_dependency("fpcalc") {
        anyhow::bail!("fpcalc (Chromaprint) is not installed or not in PATH");
    }
    if !check_external_dependency("ffprobe") {
        anyhow::bail!("ffprobe (FFmpeg) is not installed or not in PATH");
    }

    let config = DeduplicationConfig::new(
        args.scan_dir.clone(),
        args.threads,
        args.dry_run,
        args.move_to,
        args.delete,
    );
    config.validate().context("Invalid configuration")?;

    // Configure Rayon thread pool.
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()
        .ok(); // Ignore if already initialised.

    let hasher = AudioHasher;
    let files = audio::find_audio_files(&config.root_dir);

    let pb = progress_bar(files.len() as u64, "Scanning audio");

    let entries: Vec<_> = files
        .par_iter()
        .filter_map(|f| {
            let result = hasher.scan(f).ok().flatten();
            pb.inc(1);
            result
        })
        .collect();

    pb.finish_with_message("Scan complete");

    let groups = audio::group_duplicates(&entries, 2);

    let mut stats = DuplicateStats::new();
    stats.total_files = files.len();

    let dup_groups: Vec<Vec<_>> = groups.values().filter(|g| g.len() > 1).cloned().collect();
    stats.calculate_space(&dup_groups);
    stats.print_summary();

    Ok(())
}

fn run_image(args: ScanArgs) -> anyhow::Result<()> {
    setup_logging(args.verbose);

    let config = DeduplicationConfig::new(
        args.scan_dir.clone(),
        args.threads,
        args.dry_run,
        args.move_to,
        args.delete,
    );
    config.validate().context("Invalid configuration")?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()
        .ok();

    let hasher = ImageHasher::new(16);
    let files = image::find_image_files(&config.root_dir);

    let pb = progress_bar(files.len() as u64, "Scanning images");

    let metas: Vec<_> = files
        .par_iter()
        .filter_map(|f| {
            let result = hasher.scan(f).ok().flatten();
            pb.inc(1);
            result
        })
        .collect();

    pb.finish_with_message("Scan complete");

    // Exact-hash grouping first, then perceptual grouping.
    let exact_groups = image::group_by_exact_hash(&metas);
    let perceptual_groups = image::group_by_perceptual_hash(&metas, 6, true);

    let mut stats = DuplicateStats::new();
    stats.total_files = files.len();
    stats.calculate_space(&exact_groups);

    println!("Exact duplicate groups: {}", exact_groups.len());
    println!("Perceptual duplicate groups: {}", perceptual_groups.len());

    stats.print_summary();

    Ok(())
}

fn progress_bar(total: u64, prefix: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template("{prefix} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .expect("template is valid")
            .progress_chars("=> "),
    );
    pb.set_prefix(prefix.to_string());
    pb
}
