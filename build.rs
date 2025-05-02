use std::{error::Error, path::Path};

pub fn main() {
    if inner_main().is_err() {
        eprintln!("Build error: {}", inner_main().unwrap_err());

        std::process::exit(1);
    }
}

fn inner_main() -> Result<(), Box<dyn Error>> {
    let config_file_path = Path::new("hooksmith.yaml");

    if !config_file_path.exists() {
        return Err("Config file (hooksmith.yaml) for `Hooksmith` not found".into());
    }

    hooksmith::init(config_file_path)?;

    Ok(())
}
