//! Branch Operations
//!
//! Git branch-related functionality including branch information retrieval
//! and branch name formatting utilities.

use std::process::Command;

use crate::{
    errors::{GitError, Result, RonaError},
    git::commit::get_current_commit_nb,
};

/// Attempts to get the default branch name from git config.
///
/// This helper function tries to retrieve the default branch name using
/// `git config --get init.defaultBranch`. If successful, it returns the
/// branch name. If the config command fails, it returns an error with
/// the specified fallback command name for context.
///
/// # Arguments
///
/// * `fallback_command` - The command name to use in error messages when
///   the git config command fails
///
/// # Returns
///
/// * `Ok(String)` - The default branch name if successfully retrieved
/// * `Err(RonaError)` - Error with the fallback command context if config fails
fn try_get_default_branch(fallback_command: &str) -> Result<String> {
    let config_output = Command::new("git")
        .args(["config", "--get", "init.defaultBranch"])
        .output()?;

    if config_output.status.success() {
        let default_branch = String::from_utf8_lossy(&config_output.stdout)
            .trim()
            .to_string();
        Ok(default_branch)
    } else {
        // Return error with the provided fallback command context
        let error_message = String::from_utf8_lossy(&config_output.stderr);
        Err(RonaError::Git(GitError::CommandFailed {
            command: fallback_command.to_string(),
            output: error_message.to_string(),
        }))
    }
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
/// use rona::git::branch::get_current_branch;
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
        // Check if it's a freshly initialized repository (no commits yet)
        // If get_current_commit_nb() fails, it's likely a fresh repo with no HEAD
        match get_current_commit_nb() {
            Ok(0) => {
                // Fresh repository with no commits
                // Try to get the default branch name
                try_get_default_branch("git config --get init.defaultBranch")
            }
            Ok(_) => {
                // Repository has commits, so the original error wasn't due to fresh repo
                let error_message = String::from_utf8_lossy(&output.stderr);

                Err(RonaError::Git(GitError::CommandFailed {
                    command: "git rev-parse --abbrev-ref HEAD".to_string(),
                    output: error_message.to_string(),
                }))
            }
            Err(_) => {
                // Can't determine commit count, likely a fresh repo with no HEAD
                // Try to get the default branch name
                try_get_default_branch("git rev-parse --abbrev-ref HEAD")
            }
        }
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
/// use rona::git::branch::format_branch_name;
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
