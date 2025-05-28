//! # Git Operations Module
//!
//! This module provides core Git-related functionality for the Rona CLI tool. It handles
//! various Git operations, including commit management, file staging, and repository status.
//!
//! ## Key Components
//!
//! - Commit message generation and management
//! - File staging with pattern exclusion
//! - Repository status tracking
//! - Git configuration management
//!
//! ## Examples
//!
//! ```rust
//! use rona::git_related::{generate_commit_message, add_with_exclude};
//!
//! // Generate a commit message
//! let commit_type = "feat";
//! let message = generate_commit_message(commit_type)?;
//!
//! // Add files while excluding patterns
//! let patterns = vec!["*.rs", "*.tmp"];
//! add_with_exclude(&patterns)?;
//! ```
//!
//! ## Error Handling
//!
//! All Git operations return a `Result` type that can contain either the operation's
//! success value or a `RonaError`. This ensures proper error propagation and handling
//! throughout the application.

use std::{
    collections::HashSet,
    fs::{File, OpenOptions, read_to_string, write},
    io::{self, Error, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use glob::Pattern;
use regex::Regex;

use crate::{
    errors::{GitError, Result, RonaError, pretty_print_error},
    git::find_git_root,
    print_error,
    utils::{check_for_file_in_folder, find_project_root},
};

pub const COMMIT_MESSAGE_FILE_PATH: &str = "commit_message.md";
pub const COMMIT_TYPES: [&str; 4] = ["chore", "feat", "fix", "test"];
const COMMITIGNORE_FILE_PATH: &str = ".commitignore";
const GITIGNORE_FILE_PATH: &str = ".gitignore";

/// Add paths to the `.git/info/exclude` file.
///
/// # Arguments
/// * `project_root` - The path to the project root.
/// * `paths` - List of paths to add to the exclude file.
///
/// # Errors
/// * If the file cannot be read/opened/written to.
///
/// # Returns
/// * `Result<(), std::io::Error>` - Result of the operation.
pub fn add_to_git_exclude(paths: &[&str]) -> Result<()> {
    let git_root_path = find_git_root()?;

    let exclude_file = git_root_path.join("info").join("exclude");

    if !exclude_file.exists() {
        print_error(
            "No `.git/info/exclude` file found.",
            "This file is used to exclude paths from being tracked by Git.",
            "Please ensure you have a valid Git repository or submodule.",
        );

        std::process::exit(1);
    }

    // Read existing content to avoid duplicates
    let content = if exclude_file.exists() {
        read_to_string(&exclude_file)?
    } else {
        String::new()
    };

    // Parse existing paths in the file
    let existing_paths: HashSet<&str> = content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();

    // Filter paths that are not already in the file
    let paths_to_add: Vec<&str> = paths
        .iter()
        .filter(|path| !existing_paths.contains(*path))
        .copied()
        .collect();

    if paths_to_add.is_empty() {
        return Ok(());
    }

    // Open a file in `append` and `create` mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(exclude_file)?;

    // Add a marker if it's not already there
    if !content.contains("# Added by git-commit-rust") {
        if !content.is_empty() {
            writeln!(file)?;
        }
        writeln!(file, "# Added by git-commit-rust")?;
    }

    // Add each new path
    for path in paths_to_add {
        writeln!(file, "{path}")?;
    }

    Ok(())
}

/// Creates the necessary files in the project root.
///
/// # Errors
/// * If the files cannot be created.
/// * If the git add command fails.
pub fn create_needed_files() -> Result<()> {
    let project_root = find_project_root()?;
    std::env::set_current_dir(project_root)?;

    let commit_file_path = Path::new(COMMIT_MESSAGE_FILE_PATH);
    let commitignore_file_path = Path::new(COMMITIGNORE_FILE_PATH);

    if !commit_file_path.exists() {
        File::create(commit_file_path)?;
    }

    if !commitignore_file_path.exists() {
        File::create(commitignore_file_path)?;
    }

    add_to_git_exclude(&[COMMIT_MESSAGE_FILE_PATH, COMMITIGNORE_FILE_PATH])?;

    Ok(())
}

/// Formats the branch name.
/// If the branch name contains a `COMMIT_TYPES`, it will be removed.
///
/// # Arguments
/// * `commit_types` - `&[&str; 4]` - The commit types
/// * `branch` - `String` - The branch name
///
/// # Returns
/// * `String` - The formatted branch name
#[must_use]
pub fn format_branch_name(commit_types: &[&str; 4], branch: &str) -> String {
    let mut formatted_branch = branch.to_owned();

    for commit_type in commit_types {
        if formatted_branch.contains(commit_type) {
            // Remove the `/commit_type` from the branch name
            formatted_branch = formatted_branch.replace(&format!("{commit_type}/"), "");
        }
    }

    formatted_branch
}

/// Returns the current git branch.
///
/// # Errors
/// * If the git command fails
/// * If the output cannot be parsed as a string
///
/// # Returns
/// * `String` - The current git branch
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if output_str.trim().is_empty() {
        Err(RonaError::Io(Error::other("No branch found")))
    } else {
        Ok(output_str.trim().to_string())
    }
}

/// Returns the number of commits.
///
/// # Errors
/// * If the git command fails
/// * If the output cannot be parsed as a number
///
/// # Returns
/// * `u16` - The number of commits
pub fn get_current_commit_nb() -> Result<u16> {
    let branch = get_current_branch()?;

    let output = Command::new("git")
        .arg("rev-list")
        .arg("--count")
        .arg(branch)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let commit_count = output_str.trim().parse::<u16>().unwrap_or(0);

    Ok(commit_count)
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
    let deleted_files = process_deleted_files(&git_status)?;
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

    let top_level_dir = git_get_top_level_path()?;
    std::env::set_current_dir(&top_level_dir)?;

    let _ = Command::new("git")
        .arg("add")
        .args(&files_to_add)
        .args(&deleted_files)
        .output()?;

    let staged = Command::new("git")
        .args(["diff", "--cached", "--numstat"])
        .output()?;

    let staged_count = String::from_utf8_lossy(&staged.stdout).lines().count();
    let excluded_count = staged_files_len - files_to_add.len();

    println!(
        "Added {staged_count} files, deleted {deleted_files_count} and excluded {excluded_count} files for commit."
    );

    Ok(())
}

/// Returns a list of all files that appear in git status
/// (modified, untracked, staged - but not deleted)
///
/// # Errors
/// * If reading git status fails
/// * If a regex pattern fails to compile
///
/// # Returns
/// * `Vec<String>` - List of files from git status
pub fn get_status_files() -> Result<Vec<String>> {
    let status = read_git_status()?;

    // Regex to match any file in git status except deleted files
    // Matches patterns like:
    // MM file.txt
    // M  file.txt
    //  M file.txt
    // ?? file.txt
    // R  old_file.txt -> new_file.txt
    //  R old_file.txt -> new_file.txt
    let regex_rule = Regex::new(r"^[MARCU?\s][MARCU?\s]\s(.+?)(?:\s->\s(.+))?$")
        .map_err(|e| Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    // Use a HashSet to avoid duplicates
    let files: HashSet<String> = status
        .lines()
        .filter_map(|line| {
            // Skip if it's a deleted file
            if line.starts_with(" D") || line.starts_with("D ") {
                return None;
            }

            if regex_rule.is_match(line) {
                let captures = regex_rule.captures(line)?;

                // If we have a second capture group, it means we have a renamed file
                // In this case, we want to use the new filename (after the ->)
                if let Some(new_name) = captures.get(2) {
                    Some(new_name.as_str().to_string())
                } else {
                    Some(captures.get(1)?.as_str().to_string())
                }
            } else {
                println!("Error: unexpected line in git status: {line}");
                None
            }
        })
        .collect();

    let files = files.into_iter().collect();

    Ok(files)
}

/// Commits files to the git repository.
///
/// This function reads the commit message from `commit_message.md` and creates
/// a git commit with that message. Additional git arguments can be passed through.
///
/// # Arguments
/// * `args` - Additional arguments to pass to the git commit command
/// * `verbose` - Whether to print verbose output during the operation
/// * `dry_run` - If true, only show what would be committed without actually committing
///
/// # Errors
/// * If the commit message file doesn't exist
/// * If reading the commit message file fails
/// * If the git commit command fails
/// * If not in a git repository
///
/// # Examples
///
/// ```no_run
/// use rona::git_related::git_commit;
///
/// // Basic commit
/// git_commit(&[], false, false)?;
///
/// // Commit with additional git arguments
/// git_commit(&["--amend".to_string()], true, false)?;
///
/// // Dry run to preview the commit
/// git_commit(&[], false, true)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn git_commit(args: &[String], verbose: bool, dry_run: bool) -> Result<()> {
    if verbose {
        println!("Committing files...");
    }

    let project_root = find_project_root()?;
    std::env::set_current_dir(project_root)?;

    let commit_file_path = Path::new(COMMIT_MESSAGE_FILE_PATH);

    if !commit_file_path.exists() {
        return Err(RonaError::Io(Error::other("Commit message file not found")));
    }

    let file_content = read_to_string(commit_file_path)?;

    // Filter out conflicting flags
    let filtered_args: Vec<String> = args
        .iter()
        .filter(|arg| !arg.starts_with("-c") && !arg.starts_with("--commit"))
        .cloned()
        .collect();

    if dry_run {
        println!("Would commit with message:");
        println!("---");
        println!("{}", file_content.trim());
        println!("---");

        if !filtered_args.is_empty() {
            println!("With additional args: {filtered_args:?}");
        }

        return Ok(());
    }

    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(file_content)
        .args(&filtered_args)
        .output()?;

    handle_output("commit", &output, verbose)
}

/// Retrieves the top-level path of the git repository.
///
/// # Errors
/// * The git command fails.
///
/// # Returns
/// * `Result<PathBuf, Box<dyn std::error::Error>>`
pub fn git_get_top_level_path() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let git_top_level_path = PathBuf::from(stdout.trim());

    Ok(git_top_level_path)
}

/// Pushes committed changes to the remote repository.
///
/// This function executes `git push` with optional additional arguments.
/// It provides feedback on the operation's success or failure.
///
/// # Arguments
/// * `args` - Additional arguments to pass to the git push command (e.g., `--force`, `origin main`)
/// * `verbose` - Whether to print verbose output during the operation
/// * `dry_run` - If true, only show what would be pushed without actually pushing
///
/// # Errors
/// * If the git push command fails
/// * If not in a git repository
/// * If no remote repository is configured
/// * If authentication fails
///
/// # Examples
///
/// ```no_run
/// use rona::git_related::git_push;
///
/// // Basic push
/// git_push(&vec![], false, false)?;
///
/// // Push with force
/// git_push(&vec!["--force".to_string()], true, false)?;
///
/// // Push to specific remote and branch
/// git_push(&vec!["origin".to_string(), "main".to_string()], false, false)?;
///
/// // Dry run to preview the push
/// git_push(&vec![], false, true)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn git_push(args: &Vec<String>, verbose: bool, dry_run: bool) -> Result<()> {
    if verbose {
        println!("\nPushing...");
    }

    if dry_run {
        println!("Would push to remote repository");
        if !args.is_empty() {
            println!("With args: {args:?}");
        }
        return Ok(());
    }

    let output = Command::new("git").arg("push").args(args).output()?;

    handle_output("push", &output, verbose)
}

/// Prepares the commit message.
/// It creates the commit message file and empties it if it already exists.
/// It also adds the modified / added files to the commit message file.
///
/// # Errors
/// * If we cannot write to the commit message file
/// * If we cannot read the git status
/// * If we cannot process either git status or deleted files from the git status
/// * If we cannot read the commitignore file
///
/// # Arguments
/// * `commit_types` - `&str` - The commit types
/// * `verbose` - `bool` - Verbose the operation
pub fn generate_commit_message(commit_type: &str, verbose: bool) -> Result<()> {
    let commit_message_path = Path::new(COMMIT_MESSAGE_FILE_PATH);

    // Empty the file if it exists
    if commit_message_path.exists() {
        write(commit_message_path, "")?;
    }

    // Get git status info
    let git_status = read_git_status()?;
    let modified_files = process_git_status(&git_status)?;
    let deleted_files = process_deleted_files(&git_status)?;

    // Open the commit file for writing
    let mut commit_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(commit_message_path)?;

    // Write header
    write_commit_header(&mut commit_file, commit_type)?;

    // Get files to ignore
    let ignore_patterns = get_ignore_patterns()?;

    // Process modified files
    for file in modified_files {
        if !should_ignore_file(&file, &ignore_patterns)? {
            writeln!(commit_file, "- `{file}`:\n\n\t\n")?;
        }
    }

    // Process deleted files
    for file in deleted_files {
        writeln!(commit_file, "- `{file}`: deleted\n")?;
    }

    // Close the file
    commit_file.flush()?;

    if verbose {
        println!("{COMMIT_MESSAGE_FILE_PATH} created ✅ ");
    }

    Ok(())
}

/// Writes the commit header to the commit file.
///
/// # Arguments
/// * `commit_file` - The file to write to
/// * `commit_type` - The type of commit
///
/// # Errors
/// * If writing to the file fails
fn write_commit_header(commit_file: &mut File, commit_type: &str) -> Result<()> {
    let commit_number = get_current_commit_nb()? + 1;
    let branch_name = format_branch_name(&COMMIT_TYPES, &get_current_branch()?);

    writeln!(
        commit_file,
        "[{commit_number}] ({commit_type} on {branch_name})\n\n"
    )?;

    Ok(())
}

/// Gets all patterns from commitignore and gitignore files.
///
/// # Errors
/// * If reading the ignored files fails
///
/// # Returns
/// * A vector of patterns to ignore
fn get_ignore_patterns() -> Result<Vec<String>> {
    let commitignore_path = Path::new(COMMITIGNORE_FILE_PATH);

    if !commitignore_path.exists() {
        return Ok(Vec::new());
    }

    let mut patterns = process_gitignore_file()?;
    patterns.append(&mut process_gitignore_file()?);

    Ok(patterns)
}

/// Checks if a file should be ignored based on ignored patterns.
///
/// # Arguments
/// * `file` - The file to check
/// * `ignore_patterns` - Patterns to check against
///
/// # Errors
/// * If checking file paths fails
///
/// # Returns
/// * `true` if the file should be ignored, `false` otherwise
fn should_ignore_file(file: &str, ignore_patterns: &[String]) -> Result<bool> {
    // Check if the file is directly in the ignore list
    if ignore_patterns.contains(&file.to_string()) {
        return Ok(true);
    }

    // Check if the file is in a folder that's in the ignore list
    let file_path = Path::new(file);

    for item in ignore_patterns {
        let item_path = Path::new(item);

        if check_for_file_in_folder(file_path, item_path)? {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Processes the deleted files from git status output.
///
/// # Arguments
/// * `message` - The git status output string
///
/// # Errors
/// * If the extracted filenames cannot be parsed
///
/// # Returns
/// * `Result<Vec<String>, String>` - The deleted files or an error message
pub fn process_deleted_files(message: &str) -> Result<Vec<String>> {
    // Regex to match deleted files in git status output
    extract_filenames(
        message,
        r"^(?:|(?:\sD)|(?:[A-Z]D))\s{1,}([A-Za-z0-9\/_\-\.]*)$",
    )
}

/// Processes the git status.
/// It will parse the git status to prepare the git commit message.
///
/// # Arguments
/// * `message` - The git status output string
///
/// # Errors
/// * If the extracted filenames cannot be parsed
///
/// # Returns
/// * `Result<Vec<String>, String>` - The modified/added files or an error message
pub fn process_git_status(message: &str) -> Result<Vec<String>> {
    // Regex to match the modified files, added files, and renamed files
    // For renamed files, captures the new filename after '->'
    extract_filenames(message, r"^[MTARCU][A-Z\?\! ]\s(.+?)(?:\s->\s(.+))?$")
}

/// Processes the gitignore file.
///
/// # Errors
/// * If the gitignore file is not found
/// * If the gitignore file cannot be read
/// * If the gitignore file contains invalid patterns
///
/// # Returns
/// * `Result<Vec<String>, Error>` - The files and folders to ignore or an error message
pub fn process_gitignore_file() -> Result<Vec<String>> {
    // look for the gitignore file
    let gitignore_file_path = Path::new(GITIGNORE_FILE_PATH);
    //
    if !gitignore_file_path.exists() {
        return Ok(Vec::new());
    }

    let git_ignore_file_contents = read_to_string(gitignore_file_path)?;

    extract_filenames(&git_ignore_file_contents, r"^([^#]\S*)$")
}

/// Reads the git status.
///
/// # Errors
/// * If the git command fails
///
/// # Returns
/// * `Result<String>` - The git status or an error message
pub fn read_git_status() -> Result<String> {
    let args = vec!["status", "--porcelain", "-u"];
    let command = Command::new("git").args(&args).output()?;

    if command.status.success() {
        let output = String::from_utf8_lossy(&command.stdout);
        Ok(output.to_string())
    } else {
        let error_message = String::from_utf8_lossy(&command.stderr);
        Err(RonaError::Git(GitError::CommandFailed {
            command: "git rev-parse --abbrev-ref HEAD".to_string(),
            output: error_message.to_string(),
        }))
    }
}

/// Extracts filenames from a git status message using a regex pattern.
///
/// # Errors
/// * If the regex pattern is invalid
/// * If the filename cannot be captured from a line
///
/// # Returns
/// * `Result<Vec<String>>` - The extracted filenames or an error message
fn extract_filenames(message: &str, pattern: &str) -> Result<Vec<String>> {
    let regex = Regex::new(pattern).map_err(|e| GitError::InvalidStatus {
        output: format!("Failed to compile regex pattern: {e}"),
    })?;

    let mut result = Vec::new();
    for line in message.lines() {
        if regex.is_match(line) {
            if let Some(captures) = regex.captures(line) {
                // If we have a second capture group (renamed file), use that
                // Otherwise use the first capture group
                if let Some(new_name) = captures.get(2) {
                    result.push(new_name.as_str().to_string());
                } else if let Some(file_name) = captures.get(1) {
                    result.push(file_name.as_str().to_string());
                }
            }
        }
    }
    Ok(result)
}

/// Handles the output of git commands, providing consistent error handling and success messaging.
///
/// This function processes the output of git commands and:
/// - Prints success messages when verbose mode is enabled
/// - Displays command output if present
/// - Formats and prints error messages with suggestions when commands fail
///
/// # Arguments
/// * `method_name` - The name of the git command being executed (e.g., "commit", "push")
/// * `output` - The `Output` struct containing the command's stdout, stderr, and status
/// * `verbose` - Whether to print verbose output during the operation
///
/// # Returns
/// * `Result<()>` - `Ok(())` if the command succeeded, `Err(RonaError)` if it failed
/// ```
fn handle_output(method_name: &str, output: &Output, verbose: bool) -> Result<()> {
    if output.status.success() {
        if verbose {
            println!("{method_name} successful!");
        }

        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout).trim());
        }

        Ok(())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);

        println!("\n🚨 Git {method_name} failed:");
        pretty_print_error(&error_message);

        Err(RonaError::Io(Error::other(format!(
            "Git {method_name} failed"
        ))))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::Builder;

    use crate::git::find_git_root;

    use super::*;

    #[test]
    fn test_process_git_status() {
        let lines: Vec<&str> = vec![
            " M src/git_related.rs",
            "M  src/main.rs",
            "AM src/utils.rs",
            "?? src/README.md",
            "UU src/bla.rs",
            "!! src/bli.rs",
            "DD src/blo.rs",
            "R  src/old_file.rs -> src/new_file.rs",
            " R src/old_path/file.rs -> src/new_path/file.rs", // not staged so not included
            "C  src/bly.rs",
            "U  src/pae.rs",
        ];

        let modified_files = process_git_status(lines.join("\n").as_str()).unwrap();

        assert_eq!(
            modified_files,
            vec![
                "src/main.rs",
                "src/utils.rs",
                "src/bla.rs",
                "src/new_file.rs",
                "src/bly.rs",
                "src/pae.rs",
            ]
        );
    }

    #[test]
    fn test_process_deteted_files() {
        let lines: Vec<&str> = vec![
            " D src/git_related.rs",
            "D  src/main.rs",
            "AD src/utils.rs",
            "?? src/README.md",
            "UU src/bla.rs",
            "!! src/bli.rs",
            "DD src/blo.rs",
            "R  src/blu.rs",
            "C  src/bly.rs",
            "U  src/pae.rs",
        ];
        let deleted_files = process_deleted_files(lines.join("\n").as_str()).unwrap();

        assert_eq!(
            deleted_files,
            vec!["src/git_related.rs", "src/utils.rs", "src/blo.rs"]
        );
    }

    #[test]
    fn test_extract_filenames() {
        let content = "file1.txt\n#comment\nfile2.rs\n\nfile3.md";
        let pattern = r"^([^#]\S*)$";

        let result = extract_filenames(content, pattern).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains(&"file1.txt".to_string()));
        assert!(result.contains(&"file2.rs".to_string()));
        assert!(result.contains(&"file3.md".to_string()));
    }

    #[test]
    fn test_format_branch_name() {
        assert_eq!(
            format_branch_name(&COMMIT_TYPES, "feat/new-feature"),
            "new-feature"
        );
        assert_eq!(format_branch_name(&COMMIT_TYPES, "fix/bug-123"), "bug-123");
        assert_eq!(format_branch_name(&COMMIT_TYPES, "main"), "main");
        assert_eq!(
            format_branch_name(&COMMIT_TYPES, "test/add-tests"),
            "add-tests"
        );
    }

    // Helper function to initialize a git repository
    fn init_git_repo(path: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .expect("Failed to initialize git repository");
    }

    #[test]
    fn test_no_git_repository() {
        let temp_dir = Builder::new()
            .prefix("rona-test")
            .tempdir()
            .expect("Failed to create temp directory");

        // Change to temp directory and try to find git root
        std::env::set_current_dir(&temp_dir).expect("Failed to change directory");

        assert!(find_git_root().is_err());
    }

    #[test]
    fn test_basic_git_repository() {
        let temp_dir = Builder::new()
            .prefix("rona-test")
            .tempdir()
            .expect("Failed to create temp directory");

        init_git_repo(temp_dir.path());

        // Test from the repository root
        std::env::set_current_dir(&temp_dir).expect("Failed to change directory");
        let root = find_git_root().expect("Failed to find git root");
        assert!(root.ends_with(".git"));

        // Test from a subdirectory
        fs::create_dir_all(temp_dir.path().join("src/nested"))
            .expect("Failed to create nested dirs");
        std::env::set_current_dir(temp_dir.path().join("src/nested"))
            .expect("Failed to change directory");
        let root_from_nested =
            find_git_root().expect("Failed to find git root from nested directory");
        assert!(root_from_nested.ends_with(".git"));
    }

    #[test]
    fn test_corrupted_git_directory() {
        let temp_dir = Builder::new()
            .prefix("rona-test")
            .tempdir()
            .expect("Failed to create temp directory");

        // Create a .git directory but don't initialize it properly
        fs::create_dir(temp_dir.path().join(".git")).expect("Failed to create .git directory");

        std::env::set_current_dir(&temp_dir).expect("Failed to change directory");
        assert!(find_git_root().is_err());
    }
}
