//! Utility Functions Module for Rona
//!
//! This module provides common utility functions and traits used throughout the application, including
//! - Message formatting and display
//! - File and directory operations
//! - Error handling utilities
//!
//! # Message Types
//!
//! The module implements four types of messages:
//! - Error messages (🚨)
//! - Warning messages (⚠️)
//! - Success messages (✅)
//! - Info messages (ℹ️)
//!
//! # Core Features
//!
//! - Consistent message formatting
//! - File path validation and checking
//! - Project root directory detection
//! - List formatting utilities
//!
//! # Error Handling
//!
//! All file operations return `Result` types with detailed error messages
//! for proper error handling throughout the application.

use std::{
    env,
    fmt::Display,
    io::{Error as IoError, ErrorKind},
    path::{Path, PathBuf},
};

/// Trait for message types.
trait MessageType {
    /// The emoji prefix for each message type (e.g., "🚨 ERROR")
    const PREFIX: &'static str;

    /// Whether to output to stderr (true) or stdout (false)
    const TO_STDERR: bool = false;
}

// Define the message types
struct Error;
struct Warning;
struct Success;
struct Info;

// Implement the MessageType trait for each type
impl MessageType for Error {
    const PREFIX: &'static str = "🚨 ERROR";
    const TO_STDERR: bool = true;
}

impl MessageType for Warning {
    const PREFIX: &'static str = "⚠️ WARNING";
}

impl MessageType for Success {
    const PREFIX: &'static str = "✅ SUCCESS";
}

impl MessageType for Info {
    const PREFIX: &'static str = "ℹ️ INFO";
}

/// Formats a message without suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
///
/// # Returns
/// * String - The formatted message.
fn format_message<T: MessageType>(title: &str, details: &str) -> String {
    format!("{}: {title}\n\n{details}", T::PREFIX)
}

/// Formats a message with suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
/// * `suggestion` - The suggestion for the message.
///
/// # Returns
/// * String - The formatted message.
fn format_message_with_suggestion<T: MessageType>(
    title: &str,
    details: &str,
    suggestion: &str,
) -> String {
    format!("{}\n\n{suggestion}", format_message::<T>(title, details))
}

/// Prints a message without suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
///
/// # Returns
/// * String - The formatted message.
fn print_message<T: MessageType>(title: &str, details: &str) {
    let message = format_message::<T>(title, details);

    if T::TO_STDERR {
        eprintln!("{message}");
    } else {
        println!("{message}");
    }
}

/// Prints a message with suggestion.
///
/// # Arguments
/// * `title` - The title of the message.
/// * `details` - The details of the message.
/// * `suggestion` - The suggestion for resolving the message.
///
/// # Returns
/// * String - The formatted message.
fn print_message_with_suggestion<T: MessageType>(title: &str, details: &str, suggestion: &str) {
    let message = format_message_with_suggestion::<T>(title, details, suggestion);
    if T::TO_STDERR {
        eprintln!("{message}");
    } else {
        println!("{message}");
    }
}

/// Prints an error message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the error message.
/// - `details`: The details of the error message.
/// - `suggestion`: The suggestion for resolving the error.
pub fn print_error(title: &str, details: &str, suggestion: &str) {
    print_message_with_suggestion::<Error>(title, details, suggestion);
}

/// Prints a warning message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the warning message.
/// - `details`: The details of the warning message.
pub fn print_warning(title: &str, details: &str) {
    print_message::<Warning>(title, details);
}

/// Prints a success message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the success message.
/// - `details`: The details of the success message.
pub fn print_success(title: &str, details: &str) {
    print_message::<Success>(title, details);
}

/// Prints an informational message with a consistent format for user-friendly display.
///
/// # Arguments
/// - `title`: The title of the informational message.
/// - `details`: The details of the informational message.
pub fn print_info(title: &str, details: &str) {
    print_message::<Info>(title, details);
}

/// Formats a list of items with a consistent format for user-friendly display.
///
/// # Arguments
/// - `items`: The list of items to format.
///
/// # Returns
/// * String - A formatted string representation of the list.
pub fn format_list<T: Display>(items: &[T]) -> String {
    items
        .iter()
        .map(|item| format!("  - {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Checks if a file path starts with or is contained within a folder path.
///
/// # Arguments
/// * `file_path` - Path of the file to check
/// * `folder_path` - Path of the containing folder
///
/// # Errors
/// Returns an error if:
/// * The file path is invalid (empty or has an invalid parent)
/// * The folder path is invalid or empty
/// * Either path cannot be converted to a canonical form
///
/// # Returns
/// * `Ok(bool)` - True if the file is within the folder, false otherwise
/// * `Err(std::io::Error)` - If there's an error processing the paths
pub fn check_for_file_in_folder(file_path: &Path, folder_path: &Path) -> Result<bool, IoError> {
    // Validate inputs
    if file_path.as_os_str().is_empty() {
        return Err(IoError::new(ErrorKind::InvalidInput, "File path is empty"));
    }
    if folder_path.as_os_str().is_empty() {
        return Err(IoError::new(
            ErrorKind::InvalidInput,
            "Folder path is empty",
        ));
    }

    // Get the parent directory of the file
    let file_parent = file_path.parent().ok_or_else(|| {
        IoError::new(
            ErrorKind::InvalidInput,
            "Invalid file path: cannot get parent directory",
        )
    })?;

    // Check if file_path starts with folder_path
    Ok(file_parent.starts_with(folder_path))
}

/// Finds the root directory of a project by searching for a `.git` directory.
///
/// # Errors
/// * If getting the current directory fails
/// * If the project root is not found
///
/// Returns:
/// * `Ok(PathBuf)` - The path to the project root directory
/// * `Err(std::io::Error)` - If there's an error processing the paths
pub fn find_project_root() -> Result<PathBuf, IoError> {
    let mut current_dir = env::current_dir()?;

    while current_dir.parent().is_some() {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() {
            return Ok(current_dir);
        }

        let parent_dir = current_dir.parent().ok_or(IoError::new(
            ErrorKind::InvalidInput,
            "Invalid file path: cannot get parent directory",
        ))?;

        current_dir = parent_dir.to_path_buf();
    }

    Err(IoError::new(
        ErrorKind::InvalidInput,
        "Invalid file path: cannot get parent directory",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_check_for_file_in_folder() {
        // Valid cases
        assert!(check_for_file_in_folder(Path::new("src/file.rs"), Path::new("src")).unwrap());

        assert!(
            check_for_file_in_folder(Path::new("src/nested/deep/file.rs"), Path::new("src"))
                .unwrap()
        );

        assert!(!check_for_file_in_folder(Path::new("other/file.rs"), Path::new("src")).unwrap());
    }

    #[test]
    fn test_check_for_file_in_folder_errors() {
        // Empty paths
        assert!(check_for_file_in_folder(Path::new(""), Path::new("src")).is_err());

        assert!(check_for_file_in_folder(Path::new("file.txt"), Path::new("")).is_err());
    }

    #[test]
    fn test_format_list() {
        let items = vec!["item1", "item2", "item3"];
        let formatted = format_list(&items);

        assert_eq!(formatted, "  - item1\n  - item2\n  - item3");

        // Empty list
        let empty: Vec<&str> = vec![];
        assert_eq!(format_list(&empty), "");

        // Single item
        let single = vec!["item"];
        assert_eq!(format_list(&single), "  - item");
    }

    #[test]
    fn test_message_formatting() {
        // Test error message format
        let error_msg = format_message::<Error>("Test Error", "Error details");
        assert!(error_msg.contains("🚨 ERROR"));
        assert!(error_msg.contains("Test Error"));
        assert!(error_msg.contains("Error details"));

        // Test success message format
        let success_msg = format_message::<Success>("Test Success", "Success details");
        assert!(success_msg.contains("✅ SUCCESS"));
        assert!(success_msg.contains("Test Success"));
        assert!(success_msg.contains("Success details"));

        // Test with suggestion
        let error_with_suggestion = format_message_with_suggestion::<Error>(
            "Test Error",
            "Error details",
            "Try this instead",
        );
        assert!(error_with_suggestion.contains("Try this instead"));
    }
}
