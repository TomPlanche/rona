//! This build script is responsible for initializing the hooksmith configuration.
use std::path::Path;

fn main() {
    //  Rerun if the hooksmith configuration file has changed.
    //  @see https://doc.rust-lang.org/cargo/reference/build-scripts.html
    println!("cargo:rerun-if-changed=hooksmith.yaml");

    if let Err(err) = inner_main() {
        // Only fail if we're in a git repository
        // This allows building from tarballs (e.g., for Homebrew, cargo install from crates.io)
        if Path::new(".git").exists() {
            eprintln!("Error: {err}");
            std::process::exit(1);
        } else {
            eprintln!("Warning: Git hooks not initialized (not in a git repository): {err}");
        }
    }
}

fn inner_main() -> Result<(), Box<dyn std::error::Error>> {
    // Only initialize hooks if we're in a git repository
    if !Path::new(".git").exists() {
        println!("cargo:warning=Skipping git hooks initialization (not in a git repository)");
        return Ok(());
    }

    let hooksmith_config_path = Path::new("hooksmith.yaml");
    hooksmith::init(hooksmith_config_path)?;

    Ok(())
}
