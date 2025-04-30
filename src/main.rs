pub mod cli;
pub mod config;
pub mod git_related;
pub mod my_clap_theme;
pub mod utils;

use std::{error::Error, path::Path};

use cli::run;
use utils::print_error;

const GIT_ROOT: &str = ".git";

fn main() {
    if !Path::new(GIT_ROOT).exists() {
        print_error(
            "Git repository not found",
            ".git directory (or file for submodules) not found.",
            "Please ensure you're in a Git repository or submodule.",
        );

        std::process::exit(1);
    }

    if inner_main().is_err() {
        std::process::exit(1);
    }
}

fn inner_main() -> Result<(), Box<dyn Error>> {
    run()?;

    Ok(())
}
