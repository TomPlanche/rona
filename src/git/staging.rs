//! Git Staging Operations
//!
//! File staging functionality with pattern exclusion and dry-run capabilities.

use std::process::Command;

use glob::Pattern;

use crate::errors::Result;

use super::{
    repository::get_top_level_path,
    status::{get_status_files, process_deleted_files_for_staging, read_git_status},
};

/// Adds files to the git index.
///
/// # Errors
/// * If reading git status fails
/// * If adding files to git fails
/// * If getting git staged information fails
///
/// # Examples
/// ```no_run
/// use std::error::Error;
/// use glob::Pattern;
///
/// // Exclude all Rust source files
/// let patterns = vec![Pattern::new("*.rs").unwrap()];
/// git_add_with_exclude_patterns(&patterns, true)?;
///
/// // Exclude an entire directory
/// let patterns = vec![Pattern::new("target/**/*").unwrap()];
/// git_add_with_exclude_patterns(&patterns, false)?;
///
/// // Multiple exclusion patterns
/// let patterns = vec![
///     Pattern::new("*.log").unwrap(),
///     Pattern::new("temp/*").unwrap(),
///     Pattern::new("**/*.tmp").unwrap()
/// ];
/// git_add_with_exclude_patterns(&patterns, true)?;
///
/// // Complex wildcard pattern
/// let patterns = vec![Pattern::new("src/**/*_test.{rs,txt}").unwrap()];
/// git_add_with_exclude_patterns(&patterns, false)?;
///
/// // No exclusions (empty pattern list)
/// let patterns = vec![];
/// git_add_with_exclude_patterns(&patterns, true)?;
///
/// // Pattern with special characters
/// let patterns = vec![Pattern::new("[abc]*.rs").unwrap()];
/// git_add_with_exclude_patterns(&patterns, false)?;
///
/// // Error handling example
/// fn handle_git_add() -> Result<(), Box<dyn Error>> {
///     let patterns = vec![Pattern::new("*.rs")?];
///     git_add_with_exclude_patterns(&patterns, true)?;
///     Ok(())
/// }
/// ```
///
/// In these examples:
/// - `"*.rs"` excludes all Rust source files
/// - `"target/**/*"` excludes everything in the target directory and subdirectories
/// - Multiple patterns show how to exclude logs, temp files, and .tmp files
/// - `"src/**/*_test.{rs,txt}"` excludes test files with .rs or .txt extensions in src/
/// - Empty vector shows how to add all files without exclusions
/// - `"[abc]*.rs"` excludes Rust files starting with a, b, or c
/// - Error handling shows proper pattern creation with error propagation
///
/// # Arguments
/// * `exclude_patterns` - List of patterns to exclude
/// * `verbose` - Whether to print verbose output
/// * `dry_run` - If true, only show what would be added without actually staging files
pub fn git_add_with_exclude_patterns(
    exclude_patterns: &[Pattern],
    verbose: bool,
    dry_run: bool,
) -> Result<()> {
    if verbose {
        println!("Adding files...");
    }

    let git_status = read_git_status()?;
    let deleted_files = process_deleted_files_for_staging(&git_status)?;
    let deleted_files_count = deleted_files.len();

    let staged_files = get_status_files()?;
    let staged_files_len = staged_files.len();

    let files_to_add: Vec<String> = staged_files
        .into_iter()
        .filter(|file| !exclude_patterns.iter().any(|pattern| pattern.matches(file)))
        .collect();

    if files_to_add.is_empty() && deleted_files.is_empty() {
        println!("No files to add or delete");
        return Ok(());
    }

    if dry_run {
        print_dry_run_summary(&files_to_add, &deleted_files, staged_files_len);
        return Ok(());
    }

    let top_level_dir = get_top_level_path()?;
    std::env::set_current_dir(&top_level_dir)?;

    let _ = Command::new("git")
        .arg("add")
        .args(&files_to_add)
        .args(&deleted_files)
        .output()?;

    let staged = Command::new("git")
        .args(["diff", "--cached", "--numstat"])
        .output()?;

    let staged_count =
        String::from_utf8_lossy(&staged.stdout).lines().count() - deleted_files_count;
    let excluded_count = staged_files_len - files_to_add.len();

    println!(
        "Added {staged_count} files, deleted {deleted_files_count} and excluded {excluded_count} files for commit."
    );

    Ok(())
}

/// Prints a detailed summary of files that would be affected by a git add operation in dry run mode.
///
/// This function provides a clear overview of:
/// - Files that would be added to the staging area
/// - Files that would be deleted
/// - Number of files that would be excluded based on patterns
///
/// The output is formatted as follows:
/// ```
/// Would add N files:
///   + file1.txt
///   + file2.rs
/// Would delete M files:
///   - deleted_file1.txt
///   - deleted_file2.rs
/// Would exclude K files
/// ```
///
/// # Arguments
/// * `files_to_add` - List of files that would be added to the staging area
/// * `deleted_files` - List of files that would be marked as deleted
/// * `staged_files_len` - Total number of files that would be staged (including excluded ones)
/// ```
fn print_dry_run_summary(
    files_to_add: &[String],
    deleted_files: &[String],
    staged_files_len: usize,
) {
    println!("Would add {} files:", files_to_add.len());
    for file in files_to_add {
        println!("  + {file}");
    }

    println!("Would delete {} files:", deleted_files.len());
    for file in deleted_files {
        println!("  - {file}");
    }

    let excluded_files_len = staged_files_len - files_to_add.len();
    println!("Would exclude {excluded_files_len} files");
}
