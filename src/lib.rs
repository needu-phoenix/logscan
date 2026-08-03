use std::{error::Error, fs::File, io::BufReader};
use regex::Regex;

pub mod cli;
pub mod error;
pub mod commands;


pub fn run(cli: cli::Cli) -> Result<(), Box<dyn Error>> {
    let file = File::open(cli.file_name)?;
    let reader = BufReader::new(file);

    let re = Regex::new(r#"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\s+(?P<identity>\S+)\s+(?P<user>\S+)\s+\[(?P<datetime>[^\]]*)\]\s+"(?P<method>\w+)\s+(?P<path>\S+)\s+(?P<protocol>[^"]*)"\s+(?P<status>\d{3})\s+(?P<size>\d+|-)"#)?;

    match cli.command {
        cli::Commands::Filter { status } =>  commands::filter::summarize(reader, status),
        cli::Commands::Summary => commands::summary::summarize(reader, &re, cli.format)?,
        cli::Commands::Status => commands::status::summarize(reader),
        cli::Commands::Top { number } => commands::top::summarize(reader, &re, number, cli.format)?
    }
    Ok(())
}

