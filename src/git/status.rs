//! Git Status Operations
//!
//! Git status parsing and processing functionality for handling different
//! file states and contexts.

use std::{collections::HashSet, io, process::Command};

use regex::Regex;

use crate::errors::{GitError, Result, RonaError};

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
            command: "git status --porcelain -u".to_string(),
            output: error_message.to_string(),
        }))
    }
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
        .map_err(|e| RonaError::Io(io::Error::new(io::ErrorKind::InvalidData, e.to_string())))?;

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

/// Processes deleted files that need to be staged for deletion.
/// Only returns files that are deleted in the working directory but not yet staged.
///
/// # Arguments
/// * `message` - The git status output string
///
/// # Errors
/// * If the extracted filenames cannot be parsed
///
/// # Returns
/// * `Result<Vec<String>>` - Files that need to be staged for deletion
pub fn process_deleted_files_for_staging(message: &str) -> Result<Vec<String>> {
    // Regex to match files deleted in working directory but not yet staged for deletion
    // Git status format: XY filename
    // Where X = index status, Y = working tree status
    // We want files where Y = 'D' (deleted in working tree) but X ≠ 'D'
    // This includes:
    // - " D file.txt" (not in index, deleted in working tree)
    // - "MD file.txt" (modified in index, deleted in working tree)
    // - "AD file.txt" (added in index, deleted in working tree)
    // But excludes:
    // - "D  file.txt" (already staged for deletion)
    // - "DD file.txt" (deleted in both index and working tree - already staged)
    extract_filenames(message, r"^[^D]D\s+(.+)$")
}

/// Processes deleted files for commit message generation.
/// Returns all deleted files, only those that are staged or modified in the working tree.
///
/// # Arguments
/// * `message` - The git status output string
///
/// # Errors
/// * If the extracted filenames cannot be parsed
///
/// # Returns
/// * `Result<Vec<String>>` - All deleted files for the commit message
pub fn process_deleted_files_for_commit_message(message: &str) -> Result<Vec<String>> {
    // Regex to match all deleted files in git status output
    // This includes only staged deletions:
    // - " D file.txt" (deleted in the working tree only, not staged, so not included)
    // - "D  file.txt" (staged for deletion)
    // - "MD file.txt" (modified in index, deleted in the working tree)
    // - "AD file.txt" (added in index, deleted in the working tree)
    // - "DD file.txt" (deleted in both index and working tree)
    extract_filenames(message, r"^[D][D\s]\s+(.+)$")
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

// Use the shared extract_filenames function from the parent module
use super::extract_filenames;
