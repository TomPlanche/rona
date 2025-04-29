use std::{collections::HashSet, io::Error, process::Command};

use glob::Pattern;
use regex::Regex;

/// # `Add files`
/// Adds files to the git index.
///
/// ## Errors
/// * If reading git status fails
/// * If adding files to git fails
/// * If getting git staged information fails
///
/// ## Arguments
/// * `exclude_patterns` - List of patterns to exclude
/// * `verbose` - Whether to print verbose output
///
/// ## Returns
/// * `Result<(), Error>` - Result of the operation
pub fn add_files(
    exclude_patterns: &[Pattern],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("Adding files...");
    }

    let staged_files = get_status_files()?;
    let staged_files_len = staged_files.len();

    let files_to_add: Vec<String> = staged_files
        .into_iter()
        .filter(|file| {
            // If file matches any pattern, exclude it
            !exclude_patterns.iter().any(|pattern| pattern.matches(file))
        })
        .collect();

    let _ = Command::new("git")
        .arg("add")
        .args(&files_to_add)
        .output()?;

    let staged = Command::new("git")
        .args(["diff", "--cached", "--numstat"])
        .output()?;

    let staged_count = String::from_utf8_lossy(&staged.stdout).lines().count();

    let excluded_count = staged_files_len - files_to_add.len();

    if verbose {
        println!("Added {staged_count} files and excluded {excluded_count} files for commit.",);
    }

    Ok(())
}

/// # `get_status_files`
/// Returns a list of all files that appear in git status
/// (modified, untracked, staged - but not deleted)
///
/// ## Errors
/// * If reading git status fails
/// * If regex pattern fails to compile
///
/// ## Returns
/// * `Vec<String>` - List of files from git status
pub fn get_status_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let status = read_git_status()?;

    // Regex to match any file in git status except deleted files
    // Matches patterns like:
    // MM file.txt
    // M  file.txt
    //  M file.txt
    // ?? file.txt
    // But not:
    //  D file.txt
    // AD file.txt
    let regex_rule = Regex::new(r"^[MARCU? ][MARCU? ]\s(.*)$")?;

    // Use a HashSet to avoid duplicates
    let files: HashSet<String> = status
        .lines()
        .filter_map(|line| {
            // Skip if it's a deleted file
            if line.contains(" D") || line.contains("D ") {
                return None;
            }

            if regex_rule.is_match(line) {
                Some(regex_rule.captures(line)?.get(1)?.as_str().to_string())
            } else {
                println!("Error: unexpected line in git status: {line}");
                None
            }
        })
        .collect();

    Ok(files.into_iter().collect())
}

/// # `read_git_status`
/// Reads the git status.
///
/// ## Errors
/// * If the git command fails
///
/// ## Returns
/// * `Result<String, String>` - The git status or an error message
pub fn read_git_status() -> Result<String, Error> {
    // Command
    let command = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()?;

    // If the command was successful
    if command.status.success() {
        // Convert the output to a string
        let output = String::from_utf8_lossy(&command.stdout);

        Ok(output.to_string())
    } else {
        let error_message = String::from_utf8_lossy(&command.stderr);

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error_message,
        ))
    }
}
