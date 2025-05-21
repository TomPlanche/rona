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

use cli::run;
use errors::Result;
use std::process::exit;
use utils::print_error;

fn main() {
    if let Err(e) = inner_main() {
        println!("Rona error:\n{e}");

        exit(1);
    }
}

fn inner_main() -> Result<()> {
    run()?;

    Ok(())
}
