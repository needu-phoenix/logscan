use crate::{cli::OutputFormat, commands::parse_line,error:: ScanError};
use std::io::BufRead;
use serde::Serialize;
use regex::Regex;

#[derive(Serialize)]
struct FilterEntry {
    ip: String,
    path: String,
    size: u64
}

pub fn run<T: BufRead>(reader: T, status: u16, re: &Regex, format: OutputFormat) -> Result<(), ScanError> {
    match format {
        OutputFormat::Json => output_json(reader, status, re)?,
        OutputFormat::Table => output_table(reader, status, re)?
    }
    Ok(())
}

fn output_json<T: BufRead>(mut reader: T, status: u16, re: &Regex) -> Result<(), ScanError> {
    let mut line = String::new();
    let mut matches: Vec<FilterEntry> = Vec::new();
    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re) {
            if log_line.status == status {
                matches.push(FilterEntry { 
                        ip: log_line.ip, 
                        path: log_line.path, 
                        size: log_line.size 
                })
            }
        }

        line.clear();
    }

    let json = serde_json::to_string_pretty(&matches)?;

    println!("{}", json);
    Ok(())
}

fn output_table<T: BufRead>(mut reader: T, status: u16, re: &Regex) -> Result<(), ScanError> {
    let mut line = String::new();

    println!("{:<20}{:<30}{:<5}", "IP", "PATH", "SIZE");
    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re) {
            if log_line.status == status {
                println!("{:<20}{:<30}{:<5}", log_line.ip, log_line.path, log_line.size);
            }
        }

        line.clear();
    }
    Ok(())
}