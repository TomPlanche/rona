//! Git Integration Module for Rona
//!
//! This module provides comprehensive Git integration functionality, including
//! - Git repository management
//! - Commit message generation and handling
//! - Git ignore patterns management
//! - Branch operations
//!
//! # Core Features
//!
//! - Repository detection and validation
//! - Commit message file management
//! - Git status processing
//! - Branch name formatting
//! - Git command execution wrappers
//!
//! # Constants
//!
//! - `COMMIT_MESSAGE_FILE_PATH`: Path to the commit message file
//! - `COMMIT_TYPES`: Available commit types (chore, feat, fix, test)
//! - `COMMITIGNORE_FILE_PATH`: Path to the commit ignore file
//! - `GITIGNORE_FILE_PATH`: Path to the git ignore file
//!
//! # Error Handling
//!
//! All functions return `Result` types to properly handle Git-related errors
//! and provide meaningful error messages to users.

use std::{
    collections::HashSet,
    fs::{File, OpenOptions, read_to_string, write},
    io::{self, Error, Write},
    path::{Path, PathBuf},
    process::Command,
};

use glob::Pattern;
use regex::Regex;

use crate::{
    errors::{GitError, Result, RonaError},
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

/// Finds the root directory of the git repository by traversing up the directory tree
/// until it finds a .git directory or file.
///
/// # Errors
/// - If not in a git repository
/// - If unable to access directories
///
/// # Returns
/// - `Ok(PathBuf)` - Path to the git repository root
/// - `Err(GitError)` - If not in a git repository or other errors occur
pub fn find_git_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;

    if output.status.success() {
        let git_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());

        if git_root.exists() {
            Ok(git_root)
        } else {
            Err(RonaError::Git(GitError::RepositoryNotFound))
        }
    } else {
        Err(RonaError::Git(GitError::RepositoryNotFound))
    }
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
pub fn git_add_with_exclude_patterns(exclude_patterns: &[Pattern], verbose: bool) -> Result<()> {
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
        .filter(|file| {
            // If a file matches any pattern, exclude it
            !exclude_patterns.iter().any(|pattern| pattern.matches(file))
        })
        .collect();

    if files_to_add.is_empty() && deleted_files.is_empty() {
        println!("No files to add or delete");
    } else {
        let top_level_dir = git_get_top_level_path()?;
        std::env::set_current_dir(&top_level_dir)?;

        let _ = Command::new("git")
            .arg("add")
            .args(&files_to_add)
            .args(deleted_files)
            .output()?;

        let staged = Command::new("git")
            .args(["diff", "--cached", "--numstat"])
            .output()?;

        let staged_count = String::from_utf8_lossy(&staged.stdout).lines().count();

        let excluded_count = staged_files_len - files_to_add.len();

        println!(
            "Added {staged_count} files, deleted {deleted_files_count} and excluded {excluded_count} files for commit."
        );
    }

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
/// # Arguments
/// * `args` - The arguments to pass to the git commit command.
///
/// # Errors
/// * If writing commit message fails
/// * If git commit fails
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>`
pub fn git_commit(args: &Vec<String>, verbose: bool) -> Result<()> {
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
    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(file_content)
        .args(args)
        .output()?;

    if output.status.success() {
        if verbose {
            println!("Commit successful!");
        }

        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout).trim());
        }

        Ok(())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);

        println!("\n🚨 Git commit failed:");
        println!("-------------------");

        for line in error_message.lines() {
            if !line.trim().is_empty() {
                println!("{}", line.trim());
            }
        }

        Err(RonaError::Io(Error::other("Git commit failed")))
    }
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

/// Pushes the changes.
///
/// * `args` - The arguments to pass to the git push command.
/// * `verbose` - Whether to print verbose output.
///
/// # Errors
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>`
pub fn git_push(args: &Vec<String>, verbose: bool) -> Result<()> {
    if verbose {
        println!("\nPushing...");
    }

    let output = Command::new("git").arg("push").args(args).output()?;

    if output.status.success() {
        if verbose {
            println!("Push successful.");
        }

        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout).trim());
        }

        Ok(())
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);

        println!("\n 🚨 Git push failed");
        println!("-------------------");

        for line in error_message.lines() {
            if !line.trim().is_empty() {
                println!("{}", line.trim());
            }
        }

        Err(RonaError::Io(Error::other("Git commit failed")))
    }
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

    // Open commit file for writing
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
/// * If reading the ignore files fails
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

/// Checks if a file should be ignored based on ignore patterns.
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
        Err(RonaError::Git(GitError::CommandFailed(
            error_message.to_string(),
        )))
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
    let regex = Regex::new(pattern)
        .map_err(|e| GitError::InvalidStatus(format!("Failed to compile regex pattern: {e}")))?;

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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::Builder;

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
