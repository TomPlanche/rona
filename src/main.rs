//! # Rona - Git Workflow Enhancement Tool
//!
//! Rona is a command-line interface tool designed to enhance your Git workflow with powerful features
//! and intuitive commands. It simplifies common Git operations and provides additional functionality
//! for managing commits, files, and repository status.
//!
//! ## Key Features
//!
//! - Intelligent file staging with pattern exclusion
//! - Structured commit message generation
//! - Streamlined push operations
//! - Interactive commit type selection
//! - Multi-shell completion support
//!
//! ## Usage
//!
//! ```bash
//! # Initialize Rona
//! rona init [editor]
//!
//! # Add files excluding patterns
//! rona -a "*.rs"
//!
//! # Generate commit message
//! rona -g
//!
//! # Commit and push changes
//! rona -cp
//! ```
//!
//! For more detailed examples and usage instructions, see the [README.md](../README.md) file.
//!
//! # Architecture
//!
//! The application is organized into several modules:
//! - `cli`: Handles command-line interface and argument parsing
//! - `config`: Manages application configuration
//! - `errors`: Error handling and custom error types
//! - `git_related`: Contains Git-related functionality
//! - `my_clap_theme`: Custom theme for command-line output
//! - `utils`: Common utility functions
//!
//! # Error Handling
//!
//! The application implements a two-tier error handling approach:
//! 1. Initial Git repository validation
//! 2. Main application logic error handling through `Result` types
//!

pub mod cli;
pub mod config;
pub mod errors;
pub mod git;
pub mod git_related;
pub mod my_clap_theme;
pub mod performance;
pub mod utils;

use cli::run;
use errors::Result;
use std::process::exit;
use utils::print_error;

fn main() {
    if let Err(e) = inner_main() {
        eprintln!("{e}");

        exit(1);
    }
}

fn inner_main() -> Result<()> {
    run()?;

    Ok(())
}
