pub mod cli;
pub mod config;
pub mod git_related;
pub mod my_clap_theme;
pub mod utils;

use std::error::Error;

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
