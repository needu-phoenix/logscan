use crate::{cli::OutputFormat, commands::parse_line, error::ScanError};
use std::collections::{HashMap, HashSet};
use std::io::BufRead;
use serde::Serialize;
use regex::Regex;

#[derive(Debug, Serialize)]
struct Stats {
    unique_ip: HashSet<String>,
    status_count: HashMap<&'static str, usize>,
    ignored: usize,
    total_size: u64
}

pub fn run<T: BufRead>(mut reader: T, re: &Regex, format: OutputFormat) -> Result<(), ScanError> {
    let mut stats = Stats {
        unique_ip: HashSet::new(),
        status_count: HashMap::new(),
        ignored: 0,
        total_size: 0,
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
            stats.unique_ip.insert(log_line.ip);
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

fn output_table(stats: &Stats) {
    let total: usize = stats.status_count.values().sum();
    println!("{:<20}{:>10}", "Total requests:", total);
    println!("{:<20}{:>10}", "Unique IPs:", stats.unique_ip.len());
    println!("{:<20}{:>10}", "Total bytes:", stats.total_size);
    println!("{:<20}{:>10}", "Ignored lines:", stats.ignored);
    println!("\nStatus breakdown:");

    for status in ["2xx", "3xx", "4xx", "5xx", "unknown"] {
        if let Some(&count) = stats.status_count.get(status) {
            let pct = count as f64 / total as f64 * 100.0;
            println!("  {:<8}{:>10} -- ({:>4.1}%)", status, count, pct);
        }
    }
}

fn output_json(_stats: &Stats) {
}