//! Commit Operations
//!
//! Git commit-related functionality including commit counting, commit message generation,
//! and commit execution operations.

use std::{
    fs::{File, OpenOptions, read_to_string, write},
    io::Write,
    path::Path,
    process::Command,
};

use crate::{
    errors::{GitError, Result, RonaError},
    git::branch::{format_branch_name, get_current_branch},
    utils::find_project_root,
};

use super::{
    files::get_ignore_patterns,
    status::{process_deleted_files_for_commit_message, process_git_status, read_git_status},
};

pub const COMMIT_MESSAGE_FILE_PATH: &str = "commit_message.md";
pub const COMMIT_TYPES: [&str; 4] = ["chore", "feat", "fix", "test"];

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
///
/// # Returns
///
/// The total number of commits as a `u32`
///
/// # Examples
///
/// ```no_run
/// use rona::git::commit::get_current_commit_nb;
///
/// let commit_count = get_current_commit_nb()?;
/// println!("This repository has {} commits", commit_count);
///
/// // Use for commit numbering
/// let next_commit_number = commit_count + 1;
/// println!("Next commit will be #{}", next_commit_number);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn get_current_commit_nb() -> Result<u32> {
    let output = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .output()?;

    if output.status.success() {
        let commit_count_output = String::from_utf8_lossy(&output.stdout);
        let commit_count_str = commit_count_output.trim();
        let commit_count = commit_count_str.parse::<u32>().map_err(|_| {
            RonaError::Git(GitError::InvalidStatus {
                output: format!("Invalid commit count: {commit_count_str}"),
            })
        })?;

        Ok(commit_count)
    } else {
        // HEAD might not exist in a freshly initialized repository
        // Try counting all commits across all branches
        let fallback_output = Command::new("git")
            .args(["rev-list", "--count", "--all"])
            .output()?;

        if fallback_output.status.success() {
            let commit_count_output = String::from_utf8_lossy(&fallback_output.stdout);
            let commit_count_str = commit_count_output.trim();
            let commit_count = commit_count_str.parse::<u32>().map_err(|_| {
                RonaError::Git(GitError::InvalidStatus {
                    output: format!("Invalid commit count: {commit_count_str}"),
                })
            })?;

            Ok(commit_count)
        } else {
            // Return original error from the HEAD command
            let error_message = String::from_utf8_lossy(&output.stderr);

            Err(RonaError::Git(GitError::CommandFailed {
                command: "git rev-list --count HEAD".to_string(),
                output: error_message.to_string(),
            }))
        }
    }
}

/// Detects if GPG signing is available and properly configured.
///
/// This function checks multiple conditions to determine if GPG signing can be used:
/// 1. Whether a GPG signing key is configured in git
/// 2. Whether GPG is available on the system
/// 3. Whether the configured key (if any) exists in the GPG keyring
///
/// # Returns
/// * `true` if GPG signing is available and configured properly
/// * `false` if GPG signing is not available or not configured
///
/// # Panics
/// This function does not panic. All command executions are handled with proper error checking.
///
/// # Examples
///
/// ```no_run
/// use rona::git::commit::is_gpg_signing_available;
///
/// if is_gpg_signing_available() {
///     println!("GPG signing is available");
/// } else {
///     println!("GPG signing is not available, will create unsigned commit");
/// }
/// ```
#[must_use]
pub fn is_gpg_signing_available() -> bool {
    // Check if git has a signing key configured
    let git_signing_key = Command::new("git")
        .args(["config", "--get", "user.signingkey"])
        .output();

    if let Ok(output) = git_signing_key {
        if !output.status.success() || output.stdout.is_empty() {
            // No signing key configured
            return false;
        }

        let signing_key = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Check if GPG is available and the key exists
        let gpg_check = Command::new("gpg")
            .args(["--list-secret-keys", &signing_key])
            .output();

        if let Ok(gpg_output) = gpg_check {
            return gpg_output.status.success();
        }
    }

    // As a fallback, check if gpg.program is configured and accessible
    let git_gpg_program = Command::new("git")
        .args(["config", "--get", "gpg.program"])
        .output();

    if let Ok(output) = git_gpg_program
        && output.status.success()
        && !output.stdout.is_empty()
    {
        let gpg_program = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Test if the configured GPG program is available
        if let Ok(test_gpg) = Command::new(gpg_program).arg("--version").output() {
            return test_gpg.status.success();
        }
    }

    // Final fallback: check if default 'gpg' command is available
    if let Ok(default_gpg) = Command::new("gpg").arg("--version").output() {
        default_gpg.status.success()
    } else {
        false
    }
}

/// Handles dry run output for commit operations.
///
/// # Arguments
/// * `file_content` - The commit message content
/// * `unsigned` - Whether the commit should be unsigned
/// * `filtered_args` - Additional git arguments
fn handle_dry_run_output(file_content: &str, unsigned: bool, filtered_args: &[String]) {
    println!("Would commit with message:");
    println!("---");
    println!("{}", file_content.trim());
    println!("---");

    let gpg_available = is_gpg_signing_available();
    let would_sign = !unsigned && gpg_available;

    if unsigned {
        println!("Would create unsigned commit");
    } else if would_sign {
        println!("Would sign commit with -S flag");
    } else {
        println!("Would create unsigned commit (GPG signing not available)");
        if !gpg_available {
            println!("⚠️  Warning: GPG signing not available or not configured.");
            println!("   To suppress this warning, use the --unsigned (-u) flag.");
        }
    }

    if !filtered_args.is_empty() {
        println!("With additional args: {filtered_args:?}");
    }
}

/// Configures signing for git commit and displays appropriate warnings.
///
/// # Arguments
/// * `command` - The git command to configure
/// * `unsigned` - Whether signing should be disabled
/// * `verbose` - Whether to show verbose output
///
/// # Returns
/// * `bool` - Whether the commit will be signed
fn configure_commit_signing(command: &mut Command, unsigned: bool, verbose: bool) -> bool {
    let gpg_available = is_gpg_signing_available();
    let should_sign = !unsigned && gpg_available;

    if should_sign {
        command.arg("-S");
    } else if !unsigned && !gpg_available {
        println!(
            "⚠️  Warning: GPG signing not available or not configured. Creating unsigned commit."
        );
        println!("   To suppress this warning, use the --unsigned (-u) flag.");
    } else if verbose && !unsigned {
        println!("GPG signing not available, creating unsigned commit");
    }

    should_sign
}

/// Commits files to the git repository.
///
/// This function reads the commit message from `commit_message.md` and creates
/// a git commit with that message. Additional git arguments can be passed through.
/// By default, commits are signed with `-S` if GPG signing is available, unless the unsigned flag is set.
///
/// # Arguments
/// * `args` - Additional arguments to pass to the git commit command
/// * `unsigned` - If true, creates an unsigned commit (skips -S flag)
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
/// use rona::git::commit::git_commit;
///
/// // Commit with automatic GPG detection (default)
/// git_commit(&[], false, false, false)?;
///
/// // Unsigned commit
/// git_commit(&[], true, false, false)?;
///
/// // Commit with additional git arguments
/// git_commit(&["--amend".to_string()], false, true, false)?;
///
/// // Dry run to preview the commit
/// git_commit(&[], false, false, true)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn git_commit(args: &[String], unsigned: bool, verbose: bool, dry_run: bool) -> Result<()> {
    if verbose {
        println!("Committing files...");
    }

    let project_root = find_project_root()?;
    let commit_file_path = project_root.join(COMMIT_MESSAGE_FILE_PATH);

    if !commit_file_path.exists() {
        return Err(RonaError::Io(std::io::Error::other(
            "Commit message file not found",
        )));
    }

    let file_content = read_to_string(commit_file_path)?;

    // Filter out conflicting flags
    let filtered_args: Vec<String> = args
        .iter()
        .filter(|arg| !arg.starts_with("-c") && !arg.starts_with("--commit"))
        .cloned()
        .collect();

    if dry_run {
        handle_dry_run_output(&file_content, unsigned, &filtered_args);
        return Ok(());
    }

    let mut command = Command::new("git");
    command.arg("commit");

    // Configure signing and get signing status
    configure_commit_signing(&mut command, unsigned, verbose);

    command.arg("-m").arg(file_content).args(&filtered_args);

    let output = command.output()?;
    handle_output("commit", &output, verbose)
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
/// * `commit_type` - `&str` - The commit type
/// * `verbose` - `bool` - Verbose the operation
/// * `no_commit_number` - `bool` - Whether to include the commit number in the header
pub fn generate_commit_message(
    commit_type: &str,
    verbose: bool,
    no_commit_number: bool,
) -> Result<()> {
    let commit_message_path = Path::new(COMMIT_MESSAGE_FILE_PATH);

    // Empty the file if it exists
    if commit_message_path.exists() {
        write(commit_message_path, "")?;
    }

    // Get git status info
    let git_status = read_git_status()?;
    let modified_files = process_git_status(&git_status)?;
    let deleted_files = process_deleted_files_for_commit_message(&git_status)?;

    // Open the commit file for writing
    let mut commit_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(commit_message_path)?;

    // Write header
    write_commit_header(&mut commit_file, commit_type, no_commit_number)?;

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
/// * `no_commit_number` - Whether to include the commit number in the header
///
/// # Errors
/// * If writing to the file fails
fn write_commit_header(
    commit_file: &mut File,
    commit_type: &str,
    no_commit_number: bool,
) -> Result<()> {
    let branch_name = format_branch_name(&COMMIT_TYPES, &get_current_branch()?);

    if no_commit_number {
        writeln!(commit_file, "({commit_type} on {branch_name})\n\n")?;
    } else {
        let commit_number = get_current_commit_nb()? + 1;
        writeln!(
            commit_file,
            "[{commit_number}] ({commit_type} on {branch_name})\n\n"
        )?;
    }

    Ok(())
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
    use crate::utils::check_for_file_in_folder;

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

// Use the shared handle_output function from the parent module
use super::handle_output;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpg_signing_available() {
        // This test verifies that the GPG detection function doesn't panic
        // The actual result depends on the system's GPG configuration
        let _result = is_gpg_signing_available();
        // We don't assert on the result since it depends on system configuration
        // but we verify the function executes without errors
    }

    #[test]
    fn test_git_commit_dry_run_with_unsigned() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Initialize git repository
        Command::new("git")
            .current_dir(temp_path)
            .arg("init")
            .output()
            .unwrap();

        // Create commit message file
        let commit_msg = "[1] (test on main)\n\n- `test.txt`:\n\n\t\n";
        write(temp_path.join("commit_message.md"), commit_msg).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_path).unwrap();

        // Test dry run with unsigned flag - should not show warning
        let result = git_commit(&[], true, false, true);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Should succeed without errors
        assert!(result.is_ok());
    }
}
