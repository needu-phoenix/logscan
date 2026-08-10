use crate::{cli::OutputFormat, commands::parse_line, error::ScanError};
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::BufRead;

#[derive(Debug, PartialEq)]
struct Stats {
    unique_ips: HashSet<String>,
    status_count: HashMap<&'static str, usize>,
    ignored: usize,
    total_size: u64,
}

#[derive(Debug, Serialize)]
struct SummaryOutput {
    total_request: usize,
    unique_ips: usize,
    total_bytes: u64,
    ignored_lines: usize,
    status_breakdown: HashMap<&'static str, usize>,
}

pub fn run<T: BufRead>(reader: T, re: &Regex, format: OutputFormat) -> Result<(), ScanError> {
    let stats = compute(reader, re)?;
    match format {
        OutputFormat::Json => output_json(&stats)?,
        OutputFormat::Table => output_table(&stats)?,
    }

    Ok(())
}

fn compute<T: BufRead>(mut reader: T, re: &Regex) -> Result<Stats, ScanError> {
    let mut stats = Stats {
        unique_ips: HashSet::new(),
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
                _ => "unknown",
            };

            *stats.status_count.entry(key).or_insert(0) += 1;
            stats.unique_ips.insert(log_line.ip);
            stats.total_size += log_line.size;
        } else {
            stats.ignored += 1;
        }

        line.clear();
    }

    Ok(stats)
}

fn output_table(stats: &Stats) -> Result<(), ScanError> {
    let total: usize = stats.status_count.values().sum();
    println!("{:<20}{:>10}", "Total requests:", total);
    println!("{:<20}{:>10}", "Unique IPs:", stats.unique_ips.len());
    println!("{:<20}{:>10}", "Total bytes:", stats.total_size);
    println!("{:<20}{:>10}", "Ignored lines:", stats.ignored);
    println!("\nStatus breakdown:");

    for status in ["2xx", "3xx", "4xx", "5xx", "unknown"] {
        if let Some(&count) = stats.status_count.get(status) {
            let pct = count as f64 / total as f64 * 100.0;
            println!("  {:<8}{:>10} -- ({:>4.1}%)", status, count, pct);
        }
    }

    Ok(())
}

fn output_json(stats: &Stats) -> Result<(), ScanError> {
    let output = SummaryOutput {
        total_request: stats.status_count.values().sum(),
        unique_ips: stats.unique_ips.len(),
        total_bytes: stats.total_size,
        ignored_lines: stats.ignored,
        status_breakdown: stats.status_count.clone(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LOG_PATTERN;
    use std::io::Cursor;

    fn get_regex() -> Regex {
        Regex::new(LOG_PATTERN).unwrap()
    }

    #[test]
    fn count_total_request_works() {
        let data = Cursor::new([
            r#"203.0.113.11 - - [10/Oct/2025:11:42:49 +0000] "GET /api/login HTTP/1.1" 404 0 "https://twitter.com/" "python-requests/2.31.0""#, 
            r#"198.51.100.7 - - [10/Oct/2025:11:42:11 +0000] "GET /blog/post-2 HTTP/1.1" 301 - "-" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#
        ].join("\n"));

        let re = get_regex();

        let result = compute(data, &re);
        let actual: usize = result.unwrap().status_count.values().sum();

        assert_eq!(actual, 3);
    }

    #[test]
    fn count_malformed_lines_works() {
        let data = Cursor::new([
            r#"203.0.113.11 - - [10/Oct/2025:11:42:49 +0000] "/api/login HTTP/1.1" 404 0 "https://twitter.com/" "python-requests/2.31.0""#, 
            r#"198.51.100.7 - - [10/Oct/2025:11:42:11 +0000] "GET /blog/post-2 HTTP/1.1"  - "-" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"AB198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"172.16.0.24 - - [10/Oct/2025:11:37:07 +0000] "GET /sitemap.xml HTTP/1.1" 400 128 "-" "python-requests/2.31.0""#,
            r#"192.168.1.28 - alice [10/Oct/2025:11:36:47 +0000] "GET /robots.txt HTTP/1.1" 500 1547 "https://example.com/" "curl/8.4.0""#
        ].join("\n"));

        let re = get_regex();

        let result = compute(data, &re).unwrap();

        assert_eq!(result.ignored, 3);
        assert_eq!(result.status_count.values().sum::<usize>(), 2);
    }

    #[test]
    fn count_unique_ip_and_bytes() {
        let data = Cursor::new([
            r#"203.0.113.11 - - [10/Oct/2025:11:42:49 +0000] "/api/login HTTP/1.1" 404 0 "https://twitter.com/" "python-requests/2.31.0""#, 
            r#"198.51.100.7 - - [10/Oct/2025:11:42:11 +0000] "GET /blog/post-2 HTTP/1.1"  - "-" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"AB198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"172.16.0.24 - - [10/Oct/2025:11:37:07 +0000] "GET /sitemap.xml HTTP/1.1" 400 128 "-" "python-requests/2.31.0""#,
            r#"192.168.1.28 - alice [10/Oct/2025:11:36:47 +0000] "GET /robots.txt HTTP/1.1" 500 1547 "https://example.com/" "curl/8.4.0""#,
            r#"192.168.1.28 - alice [10/Oct/2025:11:36:47 +0000] "GET /robots.txt HTTP/1.1" 500 1547 "https://example.com/" "curl/8.4.0""#,
            r#"172.16.0.24 - - [10/Oct/2025:11:37:07 +0000] "GET /sitemap.xml HTTP/1.1" 400 128 "-" "python-requests/2.31.0""#,
        ].join("\n"));

        let re = get_regex();

        let result = compute(data, &re).unwrap();

        assert_eq!(result.unique_ips.len(), 2);
        assert_eq!(result.total_size, 3350);
    }
}
