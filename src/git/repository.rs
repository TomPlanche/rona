//! Repository Operations
//!
//! Core repository-level operations for Git repositories including repository detection,
//! path resolution, and basic repository information.

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
/// use rona::git::repository::find_git_root;
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
/// use rona::git::repository::get_top_level_path;
/// use std::env;
///
/// let repo_root = get_top_level_path()?;
/// env::set_current_dir(&repo_root)?;
/// println!("Changed to repository root: {}", repo_root.display());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn get_top_level_path() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let git_top_level_path = PathBuf::from(stdout.trim());

    Ok(git_top_level_path)
} 