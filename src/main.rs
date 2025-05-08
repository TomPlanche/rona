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
pub mod git_related;
pub mod my_clap_theme;
pub mod utils;

use std::{error::Error, process::exit};

use cli::run;
use git_related::find_git_root;
use utils::print_error;

fn main() {
    if find_git_root().is_err() {
        print_error(
            "Git repository not found",
            "Could not find a git repository in this directory or any parent directories.",
            "Please ensure you're in a Git repository.",
        );

        exit(1);
    }

    let result = inner_main();
    if result.is_err() {
        println!(
            "Rona error:\n{}",
            result.expect_err("Cannot unwrap Rona's error")
        );

        exit(1);
    }
}

fn inner_main() -> Result<(), Box<dyn Error>> {
    run()?;

    Ok(())
}
