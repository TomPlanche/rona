use std::{error::Error, path::Path};

use rona::{GIT_ROOT, cli::run, utils::print_error};

fn main() -> Result<(), Box<dyn Error>> {
    if !Path::new(GIT_ROOT).exists() {
        print_error(
            "Git repository not found",
            ".git directory (or file for submodules) not found.",
            "Please ensure you're in a Git repository or submodule.",
        );

        std::process::exit(1);
    }

    dotenv::dotenv().expect("Failed to load .env file");

    run()?;

    Ok(())
}
