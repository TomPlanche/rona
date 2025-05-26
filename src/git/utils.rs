//! Git Utility Functions
//!
//! Common utility functions for Git operations including repository detection,
//! path resolution, and configuration management.

use std::{path::PathBuf, process::Command};

use crate::errors::{GitError, Result, RonaError};

/// Finds the root directory of the git repository.
///
/// This function uses `git rev-parse --git-dir` to locate the `.git` directory
/// of the current repository. It works from any subdirectory within a git repository.
///
/// # Errors
///
/// Returns an error if:
/// - Not currently in a git repository
/// - The git command fails to execute
/// - The `.git` directory doesn't exist at the reported location
///
/// # Returns
///
/// - `Ok(PathBuf)` - Path to the `.git` directory
/// - `Err(RonaError::Git(GitError::RepositoryNotFound))` - If not in a git repository
///
/// # Examples
///
/// ```no_run
/// use rona::git::utils::find_git_root;
///
/// match find_git_root() {
///     Ok(git_dir) => println!("Git directory: {}", git_dir.display()),
///     Err(e) => eprintln!("Not in a git repository: {}", e),
/// }
/// ```
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

/// Retrieves the top-level path of the git repository.
///
/// This function returns the root directory of the git working tree,
/// which is the directory containing the `.git` folder. This is useful
/// for operations that need to work relative to the repository root.
///
/// # Errors
///
/// Returns an error if:
/// - Not currently in a git repository
/// - The git command fails to execute
/// - Unable to parse the command output
///
/// # Returns
///
/// The absolute path to the repository root directory
///
/// # Examples
///
/// ```no_run
/// use rona::git::utils::git_get_top_level_path;
/// use std::env;
///
/// let repo_root = git_get_top_level_path()?;
/// env::set_current_dir(&repo_root)?;
/// println!("Changed to repository root: {}", repo_root.display());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn git_get_top_level_path() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let git_top_level_path = PathBuf::from(stdout.trim());

    Ok(git_top_level_path)
}

/// Gets the current branch name.
///
/// This function returns the name of the currently checked out branch.
/// For detached HEAD states, it returns the commit hash.
///
/// # Errors
///
/// Returns an error if:
/// - Not currently in a git repository
/// - The git command fails to execute
/// - Unable to determine the current branch (e.g., in a corrupted repository)
///
/// # Returns
///
/// The name of the current branch as a `String`
///
/// # Examples
///
/// ```no_run
/// use rona::git::utils::get_current_branch;
///
/// let branch = get_current_branch()?;
/// println!("Current branch: {}", branch);
///
/// // Use in conditional logic
/// if branch == "main" {
///     println!("On main branch");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn get_current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(RonaError::Git(GitError::CommandFailed {
            command: "git rev-parse --abbrev-ref HEAD".to_string(),
            output: error_message.to_string(),
        }))
    }
}

/// Gets the total number of commits in the current branch.
///
/// This function counts all commits reachable from the current HEAD,
/// which represents the total commit count for the current branch.
/// This is useful for generating commit numbers or tracking repository activity.
///
/// # Errors
///
/// Returns an error if:
/// - Not currently in a git repository
/// - The git command fails to execute
/// - The commit count cannot be parsed as a number
/// - The commit count exceeds `u16::MAX` (65,535)
///
/// # Returns
///
/// The total number of commits as a `u16`
///
/// # Examples
///
/// ```no_run
/// use rona::git::utils::get_current_commit_nb;
///
/// let commit_count = get_current_commit_nb()?;
/// println!("This repository has {} commits", commit_count);
///
/// // Use for commit numbering
/// let next_commit_number = commit_count + 1;
/// println!("Next commit will be #{}", next_commit_number);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Note
///
/// The commit count is limited to `u16` (0-65,535). For repositories with more
/// commits, consider using a larger integer type or alternative approaches.
pub fn get_current_commit_nb() -> Result<u16> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()?;

    if output.status.success() {
        let commit_count_output = String::from_utf8_lossy(&output.stdout);
        let commit_count_str = commit_count_output.trim();
        let commit_count = commit_count_str.parse::<u16>().map_err(|_| {
            RonaError::Git(GitError::InvalidStatus {
                output: format!("Invalid commit count: {commit_count_str}"),
            })
        })?;

        Ok(commit_count)
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(RonaError::Git(GitError::CommandFailed {
            command: "git rev-list --count HEAD".to_string(),
            output: error_message.to_string(),
        }))
    }
}

/// Formats a branch name by removing commit type prefixes.
///
/// This function cleans up branch names that follow conventional naming patterns
/// like `feat/feature-name`, `fix/bug-name`, etc., by removing the commit type
/// prefix and slash, leaving just the descriptive part of the branch name.
///
/// # Arguments
///
/// * `commit_types` - An array of commit type prefixes to remove (e.g., `["feat", "fix", "chore", "test"]`)
/// * `branch` - The branch name to format
///
/// # Returns
///
/// A formatted branch name with commit type prefixes removed
///
/// # Examples
///
/// ```
/// use rona::git::utils::format_branch_name;
///
/// let commit_types = ["feat", "fix", "chore", "test"];
///
/// assert_eq!(
///     format_branch_name(&commit_types, "feat/user-authentication"),
///     "user-authentication"
/// );
///
/// assert_eq!(
///     format_branch_name(&commit_types, "fix/memory-leak"),
///     "memory-leak"
/// );
///
/// // Branch names without prefixes are unchanged
/// assert_eq!(
///     format_branch_name(&commit_types, "main"),
///     "main"
/// );
///
/// // Multiple prefixes are handled
/// assert_eq!(
///     format_branch_name(&commit_types, "feat/fix/complex-branch"),
///     "fix/complex-branch"  // Only first matching prefix is removed
/// );
/// ```
///
/// # Use Cases
///
/// This is particularly useful for:
/// - Generating clean commit messages
/// - Creating readable branch displays in UI
/// - Normalizing branch names for processing
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
