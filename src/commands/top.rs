use crate::{cli::OutputFormat, commands::parse_line, error::ScanError};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::io::BufRead;

#[derive(Debug)]
struct IpCounts {
    unique_ip: Vec<(String, usize)>,
    total_parsed: usize,
}

#[derive(Serialize)]
struct TopEntry {
    rank: usize,
    ip: String,
    requests: usize,
    percent: f64,
}

fn compute<T: BufRead>(mut reader: T, re: &Regex) -> Result<IpCounts, ScanError> {
    let mut unique_ip: HashMap<String, usize> = HashMap::new();
    let mut total_parsed: usize = 0;
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re) {
            if let Some(ip) = unique_ip.get_mut(&log_line.ip) {
                *ip += 1;
            } else {
                unique_ip.insert(log_line.ip, 1);
            }

            total_parsed += 1;
        }

        line.clear();
    }

    let mut pairs: Vec<_> = unique_ip.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let stats = IpCounts {
        unique_ip: pairs,
        total_parsed,
    };

    Ok(stats)
}

pub fn run<T: BufRead>(
    reader: T,
    re: &Regex,
    number: usize,
    format: OutputFormat,
) -> Result<(), ScanError> {
    let stats = compute(reader, re)?;
    match format {
        OutputFormat::Table => output_table(&stats, number)?,
        OutputFormat::Json => output_json(&stats, number)?,
    }

    Ok(())
}

fn output_table(stats: &IpCounts, number: usize) -> Result<(), ScanError> {
    println!(
        "{:<5}{:<15}{:<10}{:<12}",
        "RANK", "IP", "REQUESTS", "% OF TOTAL"
    );
    for (index, (ip, count)) in stats.unique_ip.iter().take(number).enumerate() {
        let pct = (*count) as f64 / stats.total_parsed as f64 * 100.0;
        println!("{:<5}{:<20}{:<10}{:<15.2}", index + 1, ip, count, pct);
    }

    Ok(())
}

fn output_json(stats: &IpCounts, number: usize) -> Result<(), ScanError> {
    let entries: Vec<TopEntry> = stats
        .unique_ip
        .iter()
        .take(number)
        .enumerate()
        .map(|(i, (ip, count))| TopEntry {
            rank: i + 1,
            ip: (*ip).clone(),
            requests: *count,
            percent: *count as f64 / stats.total_parsed as f64 * 100.0,
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&entries)?);
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
    fn ranks_by_count_with_tie_breaks() {
        let re = get_regex();
        let buffer = Cursor::new([
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.19 - - [10/Oct/2025:11:41:20 +0000] "GET /about.html HTTP/2.0" 200 458 "https://example.com/" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"198.51.100.19 - - [10/Oct/2025:11:41:20 +0000] "GET /about.html HTTP/2.0" 200 458 "https://example.com/" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"203.0.113.28 - - [10/Oct/2025:11:41:49 +0000] "POST /favicon.ico HTTP/1.1" 200 128 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15""#,
            r#"203.0.113.28 - - [10/Oct/2025:11:41:49 +0000] "POST /favicon.ico HTTP/1.1" 200 128 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:42:11 +0000] "GET /blog/post-2 HTTP/1.1" 301 - "-" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"203.0.113.11 - - [10/Oct/2025:11:42:49 +0000] "GET /api/login HTTP/1.1" 404 0 "https://twitter.com/" "python-requests/2.31.0""#
        ].join("\n"));

        let stats = compute(buffer, &re).unwrap();

        assert_eq!(stats.total_parsed, 9);
        assert_eq!(stats.unique_ip[0], ("198.51.100.7".to_string(), 4usize));
        assert_eq!(stats.unique_ip[1], ("198.51.100.19".to_string(), 2usize));
        assert_eq!(stats.unique_ip[2], ("203.0.113.28".to_string(), 2usize));
    }
}
