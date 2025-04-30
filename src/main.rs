use std::{error::Error, path::Path};

use rona::{GIT_ROOT, cli::run, utils::print_error};

fn main() {
    if !Path::new(GIT_ROOT).exists() {
        print_error(
            "Git repository not found",
            ".git directory (or file for submodules) not found.",
            "Please ensure you're in a Git repository or submodule.",
        );

        std::process::exit(1);
    }

    if let Err(e) = inner_main() {
        print_error(
            "Error occurred",
            &format!("An error occurred: {e}"),
            "Please check the error message for more details.",
        );

        std::process::exit(1);
    }
}

fn inner_main() -> Result<(), Box<dyn Error>> {
    run()?;

    Ok(())
}
