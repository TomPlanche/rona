use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions, read_to_string, write},
    io::{Error, Write},
    path::Path,
    process::Command,
};

use glob::Pattern;
use regex::Regex;

use crate::utils::{check_for_file_in_folder, print_error};

pub const COMMIT_MESSAGE_FILE_PATH: &str = "commit_message.md";
pub const COMMIT_TYPES: [&str; 4] = ["chore", "feat", "fix", "test"];
const COMMITIGNORE_FILE_PATH: &str = ".commitignore";
const GITIGNORE_FILE_PATH: &str = ".gitignore";

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

/// # `add_to_git_exclude`
/// Add paths to the `.git/info/exclude` file.
///
/// ## Arguments
/// * `project_root` - The path to the project root.
/// * `paths` - List of paths to add to the exclude file.
///
/// ## Errors
/// * If the file cannot be read/opened/written to.
///
/// ## Returns * `Result<(), std::io::Error>` - Result of the operation.
pub fn add_to_git_exclude(paths: &[&str]) -> std::io::Result<()> {
    let exclude_file = Path::new(".git").join("info").join("exclude");

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

    // Open file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(exclude_file)?;

    // Add marker if it's not already there
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

/// # `git_commit`
/// Commits files to the git repository.
///
/// ## Arguments
/// * `args` - The arguments to pass to the git commit command.
/// * `verbose` - Whether to print verbose output.
///
/// ## Errors
/// * If writing commit message fails
/// * If git commit fails
///
/// ## Returns
/// * `Result<(), Box<dyn std::error::Error>>`
pub fn git_commit(args: &Vec<String>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let commit_file_path = Path::new(COMMIT_MESSAGE_FILE_PATH);

    if commit_file_path.exists() {
        let file_content = fs::read_to_string(commit_file_path)?;

        let final_args = &["commit", "-m", file_content.as_str()];

        let command = Command::new("git").args(final_args).args(args).output();

        if let Err(e) = command {
            return Err(Box::new(Error::other(format!("Git commit failed: {e}"))));
        }

        if verbose {
            println!("Commit successful.");
        }
    } else {
        return Err(Box::new(Error::other("Commit message file not found")));
    }

    Ok(())
}

/// # `create_needed_files`
/// Creates the needed files in the project root.
///
/// ## Errors
/// * If the files cannot be created.
/// * If the git add command fails.
pub fn create_needed_files() -> Result<(), Box<dyn std::error::Error>> {
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

/// # `format_branch_name`
/// Formats the branch name.
/// If the branch name contains a `COMMIT_TYPES` it will be removed.
///
/// ## Arguments
/// * `commit_types` - `&[&str; 4]` - The commit types
/// * `branch` - `String` - The branch name
///
/// ## Returns
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

/// # `get_current_branch`
/// Returns the current git branch.
///
/// ## Errors
/// * If the git command fails
/// * If the output cannot be parsed as a string
///
/// ## Returns
/// * `String` - The current git branch
#[allow(dead_code)]
pub fn get_current_branch() -> Result<String, Error> {
    // Get the current branch
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;

    // Convert the output to a string
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// # `get_current_commit_nb`
/// Returns the number of commits.
///
/// ## Errors
/// * If the git command fails
/// * If the output cannot be parsed as a number
///
/// ## Returns
/// * `u16` - The number of commits
pub fn get_current_commit_nb() -> Result<u16, Error> {
    let output = Command::new("git")
        .arg("rev-list")
        .arg("--count")
        .arg("HEAD")
        .output()?;

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u16>()
        .map_err(|e| Error::other(format!("Failed to parse commit count: {e}")))
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

/// # `prepare_commit_msg`
/// Prepares the commit message.
/// It creates the commit message file and empty it if it already exists.
/// It also adds the modified / added files to the commit message file.
///
/// ## Errors
/// * If we cannot write to the commit message file
/// * If we cannot read the git status
/// * If we cannot process either git status or deleted files from the git status
/// * If we cannot read the commitignore file
///
/// ## Arguments
/// * `commit_types` - `&str` - The commit types
/// * `verbose` - `bool` - Verbose the operation
pub fn prepare_commit_msg(commit_type: &str, verbose: bool) -> Result<(), Error> {
    let commit_message_path = Path::new(COMMIT_MESSAGE_FILE_PATH);
    let commitignore_path = Path::new(COMMITIGNORE_FILE_PATH);

    if commit_message_path.exists() {
        // Empty the file
        write(commit_message_path, "")?;
    }

    let git_status = read_git_status()?;
    let modified_files = process_git_status(&git_status)?;
    let deleted_files = process_deleted_files(&git_status)?;

    // The commit message file
    let mut commit_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(commit_message_path)?;

    let commit_number = get_current_commit_nb()? + 1;
    let branch_name: &str = &format_branch_name(&COMMIT_TYPES, &get_current_branch()?);

    writeln!(
        commit_file,
        "[{commit_number}] ({commit_type} on {branch_name})\n\n"
    )?;

    for file in modified_files {
        // If the file is not a file in the commitignore file
        // or is not in a folder in the commitignore file
        if commitignore_path.exists() {
            let mut items_to_ignore: Vec<String> = process_gitignore_file()?;
            items_to_ignore.append(&mut process_gitignore_file()?);

            // Check if the file/folder is in the commitignore file or gitignore file
            if items_to_ignore.contains(&file) {
                // continue means skip the current iteration
                continue;
            }

            // This variable is used to call the 'continue' statement
            // just before the 'writeln!' macro.
            // I can't use the 'continue' statement directly in the for loop
            // because it will skip the next item, not file.
            let mut need_to_skip = false;

            // for each item in the commitignore file and gitignore file,
            // check for file in the folder
            // for example:
            // `data/year_2015/puzzles/` in the commitignore file can
            // exclude `data/year_2015/puzzles/day_01.md` from the commit
            // and in general `data/year_2015/puzzles/*` from the commit
            for item in items_to_ignore {
                let item_path = Path::new(&item);
                let file_path = Path::new(&file);

                if check_for_file_in_folder(file_path, item_path)? {
                    need_to_skip = true;
                }
            }

            if need_to_skip {
                // Skip the current file so the file is not added to the commit message
                continue;
            }
        }

        writeln!(commit_file, "- `{file}`:\n\n\t\n")?;
    }

    // For each deleted file
    for file in deleted_files {
        writeln!(commit_file, "- `{file}`: deleted\n")?;
    }

    // Close the file
    commit_file.flush()?;
    drop(commit_file);

    if verbose {
        // Print a message
        println!("{COMMIT_MESSAGE_FILE_PATH} created ✅ ");
    }

    Ok(())
}

/// # `process_deleted_files`
/// Processes the deleted files from git status output.
///
/// ## Arguments
/// * `message` - The git status output string
///
/// ## Errors
/// * If the extracted filenames cannot be parsed
///
/// ## Returns
/// * `Result<Vec<String>, String>` - The deleted files or an error message
pub fn process_deleted_files(message: &str) -> Result<Vec<String>, Error> {
    // Regex to match deleted files in git status output
    extract_filenames(
        message,
        r"^(?:|(?:\sD)|(?:[A-Z]D))\s{1,}([A-Za-z0-9\/_\-\.]*)$",
    )
}

/// # `process_git_status`
/// Processes the git status.
/// It will parse the git status in order to prepare the git commit message.
///
/// ## Arguments
/// * `message` - The git status output string
///
/// ## Errors
/// * If the extracted filenames cannot be parsed
///
/// ## Returns
/// * `Result<Vec<String>, String>` - The modified/added files or an error message
pub fn process_git_status(message: &str) -> Result<Vec<String>, Error> {
    // Regex to match the modified files and the added files
    extract_filenames(message, r"^[MTARCU][A-Z\?\! ]\s(.*)$")
}

/// # `process_gitignore_file`
/// Processes the gitignore file.
///
/// ## Errors
/// * If the gitignore file is not found
/// * If the gitignore file cannot be read
/// * If the gitignore file contains invalid patterns
///
/// ## Returns
/// * `Result<Vec<String>, Error>` - The files and folders to ignore or an error message
pub fn process_gitignore_file() -> Result<Vec<String>, Error> {
    // look for the gitignore file
    let gitignore_file_path = Path::new(GITIGNORE_FILE_PATH);
    //
    if !gitignore_file_path.exists() {
        return Ok(Vec::new());
    }

    let git_ignore_file_contents = fs::read_to_string(gitignore_file_path)?;

    extract_filenames(&git_ignore_file_contents, r"^([^#]\S*)$")
}

/// # `git_push`
/// Pushes the changes.
///
/// * `args` - The arguments to pass to the git push command.
/// * `verbose` - Whether to print verbose output.
///
/// ## Errors
///
/// ## Returns
/// * `Result<(), Box<dyn std::error::Error>>`
pub fn git_push(args: &Vec<String>, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    if verbose {
        println!("\nPushing...");
    }

    let command = Command::new("git").arg("push").args(args).output()?;

    if command.status.success() {
        println!("Push successful ✅");
    } else {
        let error_message = String::from_utf8_lossy(&command.stderr);

        return Err(Box::new(Error::other(error_message)));
    }

    Ok(())
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

/// # `extract_filenames`
/// Extracts filenames from a git status message using a regex pattern.
///
/// ## Errors
/// * If the regex pattern is invalid
/// * If the filename cannot be captured from a line
///
/// ## Returns
/// * `Result<Vec<String>, String>` - The extracted filenames or an error message
fn extract_filenames(message: &str, pattern: &str) -> Result<Vec<String>, Error> {
    let regex = regex::Regex::new(pattern);

    if regex.is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid regex pattern",
        ));
    }

    let regex = regex.unwrap();

    let mut result = Vec::new();
    for line in message.lines() {
        if regex.is_match(line) {
            if let Some(captures) = regex.captures(line) {
                if let Some(file_name) = captures.get(1) {
                    result.push(file_name.as_str().to_string());
                }
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_branch_name() {
        let commit_types = &["chore", "feat", "fix", "test"];

        // Test cases
        assert_eq!(
            format_branch_name(commit_types, "feat/new-feature"),
            "new-feature"
        );
        assert_eq!(format_branch_name(commit_types, "fix/bug-123"), "bug-123");
        assert_eq!(format_branch_name(commit_types, "main"), "main");
        assert_eq!(
            format_branch_name(commit_types, "chore/update-deps"),
            "update-deps"
        );
        assert_eq!(
            format_branch_name(commit_types, "test/add-tests"),
            "add-tests"
        );

        // Edge cases
        assert_eq!(format_branch_name(commit_types, ""), "");
        assert_eq!(
            format_branch_name(commit_types, "feature/stuff"),
            "feature/stuff"
        );
    }

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
            "R  src/blu.rs",
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
                "src/blu.rs",
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
}
