use crate::{cli::OutputFormat, commands::parse_line};
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead};
use serde::Serialize;
use regex::Regex;

#[derive(Debug, Serialize)]
struct Stats {
    unique_ip: HashSet<String>,
    status_count: HashMap<&'static str, usize>,
    ignored: usize,
    total_size: u64
}

pub fn summarize<T: BufRead>(mut reader: T, re: &Regex, format: OutputFormat) -> io::Result<()> {
    let mut stats = Stats {
        unique_ip: HashSet::new(),
        status_count: HashMap::new(),
        ignored: 0,
        total_size: 0
    };

    let mut line = String::new();
    
    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re) {
            let key = match log_line.status {
                200..=299 => "2xx",
                300..=399 => "3xx",
                400..=499 => "4xx",
                500..=599 => "5xx",
                _ => "unknown"
            };

            *stats.status_count.entry(key).or_insert(0) += 1;
            stats.unique_ip.insert(log_line.ip.to_string());
            stats.total_size += log_line.size;
        } else {
            stats.ignored += 1;
        }

        line.clear();
    }

    match format {
        OutputFormat::Json => output_json(&stats),
        OutputFormat::Table => output_table(&stats),
    }

    Ok(())
}

fn output_json(_stats: &Stats) {
}

fn output_table(_stats: &Stats) {

}