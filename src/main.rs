//! Rona - A Git Repository Management Tool
//!
//! This is the main entry point for the Rona application. Rona is designed to help
//! manage Git repositories with enhanced functionality and user-friendly interfaces.
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
pub mod git_related;
pub mod my_clap_theme;
pub mod utils;

use std::process::exit;

use cli::run;
use errors::Result;
use git_related::find_git_root;
use utils::print_error;

fn main() {
    // Only check for git repository if we got past the initial CLI parsing
    if let Err(e) = find_git_root() {
        eprintln!("{e}");

        exit(1);
    }

    if let Err(e) = inner_main() {
        println!("Rona error:\n{e}");

        exit(1);
    }
}

fn inner_main() -> Result<()> {
    run()?;

    Ok(())
}
