use std::{collections::HashMap, io:: BufRead};
use crate::{cli::OutputFormat, commands::parse_line, error::ScanError};
use regex::Regex;

pub fn run<T: BufRead>(mut reader: T, re: &Regex, format: OutputFormat) -> Result<(), ScanError> {
    let mut status_counts: HashMap<u16, usize> = HashMap::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re) {
            *status_counts.entry(log_line.status).or_insert(0) += 1;
        } 
        line.clear()
    }

    match format {
        OutputFormat::Json => output_json(status_counts),
        OutputFormat::Table => output_table(status_counts),
    }

    Ok(())
}

fn output_json(_counts: HashMap<u16, usize>) {

}

fn output_table(counts: HashMap<u16, usize>) {
    let mut pairs: Vec<_> = counts.iter().collect();
    pairs.sort_by(|a,b| a.0.cmp(b.0));

    println!("{:<5} {:<6} {:<6} {:<20}", "INDEX", "STATUS", "COUNT", "MEANING");

    for (index, (status, count)) in pairs.iter().enumerate() {
        println!("{:<5} {:<6} {:<6} {:<20}", index + 1, status, count, status_meaning(**status));
    }

}

fn status_meaning(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}