use std::error::Error;
use clap::Parser;
use logscan::{cli::Cli, run};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    run(args)?;
    Ok(())
}
