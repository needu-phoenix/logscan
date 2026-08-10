use regex::Regex;
use std::{fs::File, io::BufReader};

pub mod cli;
pub mod commands;
pub mod error;

pub fn run(cli: cli::Cli) -> Result<(), error::ScanError> {
    let file = File::open(&cli.file_name).map_err(|source| error::ScanError::Io {
        path: cli.file_name,
        source,
    })?;
    let reader = BufReader::new(file);

    let re = Regex::new(
        r#"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\s+(?P<identity>\S+)\s+(?P<user>\S+)\s+\[(?P<datetime>[^\]]*)\]\s+"(?P<method>\w+)\s+(?P<path>\S+)\s+(?P<protocol>[^"]*)"\s+(?P<status>\d{3})\s+(?P<size>\d+|-)"#,
    )?;

    match cli.command {
        cli::Commands::Filter { status } => commands::filter::run(reader, status, &re, cli.format)?,
        cli::Commands::Summary => commands::summary::run(reader, &re, cli.format)?,
        cli::Commands::Status => commands::status::run(reader, &re, cli.format)?,
        cli::Commands::Top { number } => commands::top::run(reader, &re, number, cli.format)?,
    }
    Ok(())
}
