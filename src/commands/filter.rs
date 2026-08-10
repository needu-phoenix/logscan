use crate::{cli::OutputFormat, commands::parse_line, error::ScanError};
use regex::Regex;
use serde::Serialize;
use std::io::BufRead;

#[derive(Serialize, Debug)]
struct FilterEntry {
    ip: String,
    path: String,
    size: u64,
}

fn compute<T: BufRead>(
    mut reader: T,
    re: &Regex,
    status: u16,
) -> Result<Vec<FilterEntry>, ScanError> {
    let mut line = String::new();
    let mut matches: Vec<FilterEntry> = Vec::new();
    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re)
            && log_line.status == status
        {
            matches.push(FilterEntry {
                ip: log_line.ip,
                path: log_line.path,
                size: log_line.size,
            })
        }

        line.clear();
    }
    Ok(matches)
}

fn output_json<T: BufRead>(reader: T, status: u16, re: &Regex) -> Result<(), ScanError> {
    let matches = compute(reader, re, status)?;
    let json = serde_json::to_string_pretty(&matches)?;
    println!("{}", json);
    Ok(())
}

fn output_table<T: BufRead>(mut reader: T, status: u16, re: &Regex) -> Result<(), ScanError> {
    let mut line = String::new();

    println!("{:<20}{:<30}{:<5}", "IP", "PATH", "SIZE");
    while reader.read_line(&mut line)? > 0 {
        if let Some(log_line) = parse_line(&line, re)
            && log_line.status == status
        {
            println!(
                "{:<20}{:<30}{:<5}",
                log_line.ip, log_line.path, log_line.size
            );
        }

        line.clear();
    }
    Ok(())
}

pub fn run<T: BufRead>(
    reader: T,
    status: u16,
    re: &Regex,
    format: OutputFormat,
) -> Result<(), ScanError> {
    match format {
        OutputFormat::Json => output_json(reader, status, re)?,
        OutputFormat::Table => output_table(reader, status, re)?,
    }
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
    fn return_matching_lines() {
        let re = get_regex();
        let buffer = Cursor::new([
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.19 - - [10/Oct/2025:11:41:20 +0000] "GET /about.html HTTP/2.0" 404 458 "https://example.com/" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"198.51.100.19 - - [10/Oct/2025:11:41:20 +0000] "GET /about.html HTTP/2.0" 200 458 "https://example.com/" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"203.0.113.28 - - [10/Oct/2025:11:41:49 +0000] "POST /favicon.ico HTTP/1.1" 404 128 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15""#,
            r#"203.0.113.28 - - [10/Oct/2025:11:41:49 +0000] "POST /favicon.ico HTTP/1.1" 200 128 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:42:11 +0000] "GET /blog/post-2 HTTP/1.1" 301 - "-" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"203.0.113.11 - - [10/Oct/2025:11:42:49 +0000] "GET /api/login HTTP/1.1" 404 0 "https://twitter.com/" "python-requests/2.31.0""#
        ].join("\n"));

        let four_o_four = compute(buffer.clone(), &re, 404).unwrap();
        assert_eq!(four_o_four.len(), 3);

        let two_o_o = compute(buffer, &re, 200).unwrap();
        assert_eq!(two_o_o.len(), 5);
    }

    #[test]
    fn empty_buffer_returns_empty() {
        let re = get_regex();
        let buffer = Cursor::new([String::new()].join("\n"));

        let empty = compute(buffer.clone(), &re, 404).unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn malformed_line_skipped() {
        let re = get_regex();
        let buffer = Cursor::new([
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:40:43 +0000] "POST /app.js HTTP/1.1" 200 234 "https://example.com/" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36""#,
            r#"198.51.100.19 - - [10/Oct/2025:11:41:20 +0000] "GET /about.html HTTP/2.0" 404 458 "https://example.com/" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"198.51.100.19 - - [10/Oct/2025:11:41:20 +0000] "GET /about.html HTTP/2.0" 200 458 "https://example.com/" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#".0.113.28 - - [10/Oct/2025:11:41:49 +0000] "POST /favicon.ico HTTP/1.1" 404 128 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15""#,
            r#"203.0.113.28 - - [10/Oct/2025:11:41:49 +0000] "POST /favicon.ico HTTP/1.1" 200 128 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15""#,
            r#"198.51.100.7 - - [10/Oct/2025:11:42:11 +0000] "GET /blog/post-2 HTTP/1.1" 301 - "-" "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)""#,
            r#"203.0.113.11 - - [10/Oct/2025:11:42:49 +0000] "GET /api/login HTTP/1.1" 404 0 "https://twitter.com/" "python-requests/2.31.0""#
        ].join("\n"));

        let four_o_four = compute(buffer.clone(), &re, 404).unwrap();
        assert_eq!(four_o_four.len(), 2);
    }
}
