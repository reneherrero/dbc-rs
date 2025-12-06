//! Real-world DBC file testing using files from multiple sources.
//!
//! This test suite validates the library against actual DBC files from the
//! automotive industry. It tests parsing, validation, and optionally round-trip
//! (parse -> save -> parse again).
//!
//! # Sources
//!
//! The test suite supports DBC files from:
//! - **commaai/opendbc**: Large collection of automotive DBC files
//! - **WURacing/DBC**: Racing context CAN database files
//! - **CSS Electronics**: Demo J1939 and other protocol DBC files
//!
//! # Usage
//!
//! These tests are optional and require the `real-world-tests` feature:
//!
//! ```bash
//! # Run all real-world tests (recommended - shows unified summary)
//! cargo test test_all_real_world_files -- --ignored
//! ```
//!
//! **Note**: Tests run sequentially by default in Rust. The output format shows:
//! - `-- Summary --`: Statistics for each DBC source (commaai/opendbc, WURacing/DBC, CSS Electronics)
//! - `-- Errors --`: Detailed error listings grouped by source and folder path
//!
//! # Setup
//!
//! The tests will automatically download repositories if they're not present.
//! Alternatively, you can set up git submodules:
//!
//! ```bash
//! git submodule add https://github.com/commaai/opendbc.git tests/opendbc
//! git submodule add https://github.com/WURacing/DBC.git tests/wuracing
//! ```
//!
//! For CSS Electronics files, you can set the `CSS_DBC_PATH` environment variable
//! to point to a directory containing their DBC files.

#[cfg(feature = "std")]
use dbc_rs::Dbc;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Statistics collected during real-world testing.
#[derive(Debug, Default)]
struct TestStats {
    total_files: usize,
    parsed_successfully: usize,
    parse_failed: usize,
    round_trip_successful: usize,
    round_trip_failed: usize,
    total_messages: usize,
    total_signals: usize,
    // Map from folder path to list of (filename, error) pairs
    parse_errors: BTreeMap<String, Vec<(String, String)>>,
    round_trip_errors: BTreeMap<String, Vec<(String, String)>>,
}

impl TestStats {
    /// Print summary statistics for a source
    fn print_summary_section(&self, source_name: &str) {
        eprintln!("\n* {}", source_name);
        eprintln!("DBC files found: {}", self.total_files);
        eprintln!(
            "Parse succeeded: {} ({:.1}%)",
            self.parsed_successfully,
            100.0 * self.parsed_successfully as f64 / self.total_files.max(1) as f64
        );
        eprintln!(
            "Parse failed: {} ({:.1}%)",
            self.parse_failed,
            100.0 * self.parse_failed as f64 / self.total_files.max(1) as f64
        );
        eprintln!(
            "Round-trip succeeded: {} ({:.1}%)",
            self.round_trip_successful,
            100.0 * self.round_trip_successful as f64 / self.parsed_successfully.max(1) as f64
        );
        eprintln!(
            "Round-trip failed: {} ({:.1}%)",
            self.round_trip_failed,
            100.0 * self.round_trip_failed as f64 / self.parsed_successfully.max(1) as f64
        );
    }

    /// Print errors for a source
    fn print_errors_section(&self, source_name: &str) {
        let total_parse_errors: usize = self.parse_errors.values().map(|v| v.len()).sum();
        let total_round_trip_errors: usize = self.round_trip_errors.values().map(|v| v.len()).sum();

        if total_parse_errors == 0 && total_round_trip_errors == 0 {
            return;
        }

        eprintln!("\n* {}", source_name);

        // Get all unique folders from both error maps
        let mut all_folders: BTreeMap<String, ()> = BTreeMap::new();
        for folder in self.parse_errors.keys() {
            all_folders.insert(folder.clone(), ());
        }
        for folder in self.round_trip_errors.keys() {
            all_folders.insert(folder.clone(), ());
        }

        // For each folder, show parse failures first, then round-trip failures
        for folder in all_folders.keys() {
            let has_parse_errors = self.parse_errors.contains_key(folder);
            let has_round_trip_errors = self.round_trip_errors.contains_key(folder);

            if has_parse_errors || has_round_trip_errors {
                eprintln!("\nDBC path: {}", folder);

                // Show parse failures for this folder
                if let Some(parse_errors) = self.parse_errors.get(folder) {
                    if !parse_errors.is_empty() {
                        eprintln!("Parse errors: {}", parse_errors.len());
                        for (filename, error) in parse_errors {
                            eprintln!("- {}: {}", filename, error);
                        }
                    }
                }

                // Show round-trip failures for this folder
                if let Some(round_trip_errors) = self.round_trip_errors.get(folder) {
                    if !round_trip_errors.is_empty() {
                        eprintln!("Round-trip errors: {}", round_trip_errors.len());
                        for (filename, error) in round_trip_errors {
                            eprintln!("- {}: {}", filename, error);
                        }
                    }
                }
            }
        }
    }
}

/// Get the path to opendbc files.
///
/// Checks for:
/// 1. Git submodule at `tests/opendbc/`
/// 2. Downloaded files at `tests/opendbc/`
/// 3. Environment variable `OPENDBC_PATH`
fn get_opendbc_path() -> Option<PathBuf> {
    // Check for git submodule or downloaded files
    let submodule_path = PathBuf::from("tests/opendbc");
    if submodule_path.exists() {
        return Some(submodule_path);
    }

    // Check environment variable
    if let Ok(env_path) = std::env::var("OPENDBC_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Get the path to WURacing DBC files.
///
/// Checks for:
/// 1. Git submodule at `tests/wuracing/`
/// 2. Downloaded files at `tests/wuracing/`
/// 3. Environment variable `WURACING_DBC_PATH`
fn get_wuracing_path() -> Option<PathBuf> {
    // Check for git submodule or downloaded files
    let submodule_path = PathBuf::from("tests/wuracing");
    if submodule_path.exists() {
        return Some(submodule_path);
    }

    // Check environment variable
    if let Ok(env_path) = std::env::var("WURACING_DBC_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Get the path to CSS Electronics DBC files.
///
/// Checks for:
/// 1. Directory at `tests/css_dbc/`
/// 2. Environment variable `CSS_DBC_PATH`
fn get_css_dbc_path() -> Option<PathBuf> {
    // Check for local directory
    let local_path = PathBuf::from("tests/css_dbc");
    if local_path.exists() {
        return Some(local_path);
    }

    // Check environment variable
    if let Ok(env_path) = std::env::var("CSS_DBC_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Find all DBC files in a directory recursively.
fn find_dbc_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip .git directories
                if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                    continue;
                }
                files.extend(find_dbc_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("dbc") {
                files.push(path);
            }
        }
    }

    files
}

/// Test parsing a single DBC file.
#[cfg(feature = "std")]
fn test_parse_file(path: &Path, stats: &mut TestStats) {
    stats.total_files += 1;

    // Extract folder path and filename
    let folder_path = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

    // Try to parse directly from file
    match Dbc::from_file(path) {
        Ok(dbc) => {
            stats.parsed_successfully += 1;
            stats.total_messages += dbc.messages().len();
            stats.total_signals += dbc.messages().iter().map(|m| m.signals().len()).sum::<usize>();

            // Test round-trip: parse -> save -> parse again
            let saved = dbc.to_dbc_string();
            match Dbc::parse(&saved) {
                Ok(_) => {
                    stats.round_trip_successful += 1;
                }
                Err(e) => {
                    stats.round_trip_failed += 1;
                    stats
                        .round_trip_errors
                        .entry(folder_path)
                        .or_default()
                        .push((filename, format!("Round-trip failed: {}", e)));
                }
            }
        }
        Err(e) => {
            stats.parse_failed += 1;
            stats
                .parse_errors
                .entry(folder_path)
                .or_default()
                .push((filename, format!("{}", e)));
        }
    }
}

/// Download opendbc files using git (if git is available).
fn download_opendbc() -> Result<PathBuf, String> {
    let target_dir = PathBuf::from("tests/opendbc");

    // Check if already exists
    if target_dir.exists() {
        return Ok(target_dir);
    }

    // Try to clone
    println!("Downloading opendbc repository...");
    let output = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/commaai/opendbc.git",
            target_dir.to_str().unwrap(),
        ])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            println!("Successfully downloaded opendbc files");
            Ok(target_dir)
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            Err(format!("git clone failed: {}", stderr))
        }
        Err(e) => Err(format!("Failed to run git: {}", e)),
    }
}

/// Download WURacing DBC files using git (if git is available).
fn download_wuracing() -> Result<PathBuf, String> {
    let target_dir = PathBuf::from("tests/wuracing");

    // Check if already exists
    if target_dir.exists() {
        return Ok(target_dir);
    }

    // Try to clone
    println!("Downloading WURacing DBC repository...");
    let output = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/WURacing/DBC.git",
            target_dir.to_str().unwrap(),
        ])
        .output();

    match output {
        Ok(result) if result.status.success() => {
            println!("Successfully downloaded WURacing DBC files");
            Ok(target_dir)
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            Err(format!("git clone failed: {}", stderr))
        }
        Err(e) => Err(format!("Failed to run git: {}", e)),
    }
}

/// Test DBC files from a specific source.
/// Returns statistics for the source.
fn test_dbc_source(_name: &str, path: PathBuf) -> TestStats {
    #[cfg(feature = "std")]
    let mut stats = TestStats::default();
    #[cfg(not(feature = "std"))]
    let stats = TestStats::default();

    // Find all DBC files
    let dbc_files = find_dbc_files(&path);

    if dbc_files.is_empty() {
        return stats;
    }

    // Test each file
    #[cfg(feature = "std")]
    {
        for file_path in &dbc_files {
            test_parse_file(file_path, &mut stats);
        }
    }

    stats
}

#[test]
#[ignore = "uses unproven sources"]
fn test_all_real_world_files() {
    let mut opendbc_stats: Option<TestStats> = None;
    let mut wuracing_stats: Option<TestStats> = None;
    let mut css_stats: Option<TestStats> = None;

    // Test opendbc files
    if let Some(path) = get_opendbc_path() {
        opendbc_stats = Some(test_dbc_source("commaai/opendbc", path));
    } else {
        match download_opendbc() {
            Ok(path) => {
                opendbc_stats = Some(test_dbc_source("commaai/opendbc", path));
            }
            Err(e) => {
                eprintln!("Warning: Could not find or download opendbc files: {}", e);
                eprintln!("  Set OPENDBC_PATH environment variable or run:");
                eprintln!(
                    "  git submodule add https://github.com/commaai/opendbc.git tests/opendbc"
                );
            }
        }
    }

    // Test WURacing files
    if let Some(path) = get_wuracing_path() {
        wuracing_stats = Some(test_dbc_source("WURacing/DBC", path));
    } else {
        match download_wuracing() {
            Ok(path) => {
                wuracing_stats = Some(test_dbc_source("WURacing/DBC", path));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not find or download WURacing DBC files: {}",
                    e
                );
                eprintln!("  Set WURACING_DBC_PATH environment variable or run:");
                eprintln!("  git submodule add https://github.com/WURacing/DBC.git tests/wuracing");
            }
        }
    }

    // Test CSS Electronics files
    if let Some(path) = get_css_dbc_path() {
        css_stats = Some(test_dbc_source("CSS Electronics", path));
    } else {
        eprintln!("Warning: CSS Electronics DBC files not found");
        eprintln!("  Set CSS_DBC_PATH environment variable or place files in tests/css_dbc/");
        eprintln!(
            "  Download from: https://www.csselectronics.com/pages/can-dbc-file-database-intro"
        );
    }

    if opendbc_stats.is_none() && wuracing_stats.is_none() && css_stats.is_none() {
        eprintln!("\nNo DBC file sources available. Skipping real-world tests.");
        eprintln!("To enable tests, set up at least one source:");
        eprintln!("  1. OPENDBC_PATH or git submodule for commaai/opendbc");
        eprintln!("  2. WURACING_DBC_PATH or git submodule for WURacing/DBC");
        eprintln!("  3. CSS_DBC_PATH for CSS Electronics files");
        return;
    }

    // Print unified output in the new format
    eprintln!("\n=== Real-World Test ===");
    eprintln!("\n-- Summary --");

    if let Some(ref stats) = opendbc_stats {
        stats.print_summary_section("commaai/opendbc");
    }

    if let Some(ref stats) = wuracing_stats {
        stats.print_summary_section("WURacing/DBC");
    }

    if let Some(ref stats) = css_stats {
        stats.print_summary_section("CSS Electronics");
    }

    // Check if there are any errors to report
    let has_errors = opendbc_stats
        .as_ref()
        .map(|s| !s.parse_errors.is_empty() || !s.round_trip_errors.is_empty())
        .unwrap_or(false)
        || wuracing_stats
            .as_ref()
            .map(|s| !s.parse_errors.is_empty() || !s.round_trip_errors.is_empty())
            .unwrap_or(false)
        || css_stats
            .as_ref()
            .map(|s| !s.parse_errors.is_empty() || !s.round_trip_errors.is_empty())
            .unwrap_or(false);

    if has_errors {
        eprintln!("\n-- Errors --");

        if let Some(ref stats) = opendbc_stats {
            stats.print_errors_section("commaai/opendbc");
        }

        if let Some(ref stats) = wuracing_stats {
            stats.print_errors_section("WURacing/DBC");
        }

        if let Some(ref stats) = css_stats {
            stats.print_errors_section("CSS Electronics");
        }
    }

    // Calculate overall statistics for assertions
    let total_parsed: usize = opendbc_stats.as_ref().map(|s| s.parsed_successfully).unwrap_or(0)
        + wuracing_stats.as_ref().map(|s| s.parsed_successfully).unwrap_or(0)
        + css_stats.as_ref().map(|s| s.parsed_successfully).unwrap_or(0);

    let total_files: usize = opendbc_stats.as_ref().map(|s| s.total_files).unwrap_or(0)
        + wuracing_stats.as_ref().map(|s| s.total_files).unwrap_or(0)
        + css_stats.as_ref().map(|s| s.total_files).unwrap_or(0);

    // Assert that we parsed at least some files successfully
    // (We don't require 100% success since some files may have unsupported features)
    assert!(
        total_parsed > 0,
        "Expected to parse at least one DBC file, but all {} files failed",
        total_files
    );

    // Warn if overall success rate is very low
    let success_rate = total_parsed as f64 / total_files.max(1) as f64;
    if success_rate < 0.5 {
        eprintln!(
            "\n⚠️  Warning: Overall success rate is only {:.1}%. \
             This may indicate compatibility issues.",
            success_rate * 100.0
        );
    }
}
