//! Git Operations Module
//!
//! This module provides organized Git-related functionality for the Rona CLI tool.
//! It's organized into focused submodules for better maintainability and clear separation of concerns.
//!
//! ## Submodules
//!
//! - [`repository`] - Core repository operations (finding git root, top level path)
//! - [`branch`] - Branch operations (current branch, branch name formatting)
//! - [`commit`] - Commit operations (commit counting, committing, commit message generation)
//! - [`status`] - Git status parsing and processing
//! - [`staging`] - File staging operations with pattern exclusion
//! - [`remote`] - Remote operations (git push)
//! - [`files`] - File and exclusion handling utilities

pub mod branch;
pub mod commit;
pub mod files;
pub mod remote;
pub mod repository;
pub mod staging;
pub mod status;

// Re-export commonly used functions for convenience
pub use branch::{format_branch_name, get_current_branch};
pub use commit::{
    COMMIT_MESSAGE_FILE_PATH, COMMIT_TYPES, generate_commit_message, get_current_commit_nb,
    git_commit,
};
pub use files::create_needed_files;
pub use remote::git_push;
pub use repository::find_git_root;
pub use staging::git_add_with_exclude_patterns;
pub use status::get_status_files;
