use crate::{commands::parse_line, cli::OutputFormat, error::ScanError};
use std::collections::HashMap;
use std::io::BufRead;
use serde::Serialize;
use regex::Regex;


#[derive(Debug, Serialize)]
struct IpCounts {
    unique_ip: HashMap<String, usize>,
    ignored: usize,
}

pub fn run<T: BufRead>(mut reader: T, re: &Regex, number: usize, format: OutputFormat) -> Result<(), ScanError> {
    let mut stats = IpCounts {
        unique_ip: HashMap::new(),
        ignored: 0
    };

    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re) {
            if let Some(ip) = stats.unique_ip.get_mut(&log_line.ip) {
                *ip += 1
            } else {
                stats.unique_ip.insert(log_line.ip, 1);
            }
        } else {
            stats.ignored += 1
        }

        line.clear();
    }

    match format {
        OutputFormat::Table => output_table(&stats, number),
        OutputFormat::Json => output_json(&stats, number),
    }

    Ok(())
}

fn output_table(stats: &IpCounts, number: usize) {
    let total = stats.unique_ip.values().sum::<usize>();
    let mut pairs: Vec<_> = stats.unique_ip.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    
    println!("{:<5}{:<15}{:<10}{:<12}","RANK", "IP", "REQUESTS", "% OF TOTAL");
    for (index, (ip, count)) in pairs.iter().take(number).enumerate() {
        let pct = (**count) as f64 / total as f64 * 100.0;
        println!("{:<5}{:<20}{:<10}{:<15.2}",index + 1, ip, count, pct);
    }
}

fn output_json(_stats: &IpCounts, _number: usize) {

}