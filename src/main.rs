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
