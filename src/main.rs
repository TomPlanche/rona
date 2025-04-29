use std::error::Error;

use rona::cli::run;

fn main() -> Result<(), Box<dyn Error>> {
    run()?;

    Ok(())
}
