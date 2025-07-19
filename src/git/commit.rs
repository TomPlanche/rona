//! Commit Operations
//!
//! Git commit-related functionality including commit counting, commit message generation,
//! and commit execution operations.

use std::{
    fs::{File, OpenOptions, read_to_string, write},
    io::Write,
    path::Path,
    process::{Command, Output},
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
        let error_message = String::from_utf8_lossy(&output.stderr);
        Err(RonaError::Git(GitError::CommandFailed {
            command: "git rev-list --count HEAD".to_string(),
            output: error_message.to_string(),
        }))
    }
}

/// Commits files to the git repository.
///
/// This function reads the commit message from `commit_message.md` and creates
/// a git commit with that message. Additional git arguments can be passed through.
/// By default, commits are signed with `-S` unless the unsigned flag is set.
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
/// // Basic signed commit (default)
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
    std::env::set_current_dir(project_root)?;

    let commit_file_path = Path::new(COMMIT_MESSAGE_FILE_PATH);

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
        println!("Would commit with message:");
        println!("---");
        println!("{}", file_content.trim());
        println!("---");

        if unsigned {
            println!("Would create unsigned commit");
        } else {
            println!("Would sign commit with -S flag");
        }

        if !filtered_args.is_empty() {
            println!("With additional args: {filtered_args:?}");
        }

        return Ok(());
    }

    let mut command = Command::new("git");
    command.arg("commit");

    // Add -S flag for signed commits by default, unless unsigned is requested
    if !unsigned {
        command.arg("-S");
    }

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
#[doc(hidden)]
fn handle_output(method_name: &str, output: &Output, verbose: bool) -> Result<()> {
    use crate::errors::pretty_print_error;

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

        Err(RonaError::Io(std::io::Error::other(format!(
            "Git {method_name} failed"
        ))))
    }
}
