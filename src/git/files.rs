//! File and Exclusion Handling
//!
//! Git file operations including exclusion patterns, ignore file processing,
//! and file management utilities.

use std::{
    collections::HashSet,
    fs::{File, OpenOptions, read_to_string},
    io::Write,
    path::Path,
};

use regex::Regex;

use crate::{
    errors::Result,
    git::{COMMIT_MESSAGE_FILE_PATH, find_git_root},
    print_error,
    utils::find_project_root,
};

const COMMITIGNORE_FILE_PATH: &str = ".commitignore";
const GITIGNORE_FILE_PATH: &str = ".gitignore";

/// Add paths to the `.git/info/exclude` file.
///
/// # Arguments
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

/// Gets all patterns from commitignore and gitignore files.
///
/// # Errors
/// * If reading the ignored files fails
///
/// # Returns
/// * A vector of patterns to ignore
pub fn get_ignore_patterns() -> Result<Vec<String>> {
    let commitignore_path = Path::new(COMMITIGNORE_FILE_PATH);

    if !commitignore_path.exists() {
        return Ok(Vec::new());
    }

    let mut patterns = process_gitignore_file()?;
    patterns.append(&mut process_gitignore_file()?);

    Ok(patterns)
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

/// Extracts filenames from a git status message using a regex pattern.
///
/// # Errors
/// * If the regex pattern is invalid
/// * If the filename cannot be captured from a line
///
/// # Returns
/// * `Result<Vec<String>>` - The extracted filenames or an error message
#[doc(hidden)]
fn extract_filenames(message: &str, pattern: &str) -> Result<Vec<String>> {
    use crate::errors::{GitError, RonaError};

    let regex = Regex::new(pattern).map_err(|e| {
        RonaError::Git(GitError::InvalidStatus {
            output: format!("Failed to compile regex pattern: {e}"),
        })
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
