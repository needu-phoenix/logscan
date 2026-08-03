use regex::Regex;

pub mod summary;
pub mod status;
pub mod filter;
pub mod top;


pub struct LogLine {
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub size: u64
}

pub fn parse_line(line: &str, re: &Regex) -> Option<LogLine> {
    let cap = re.captures(line)?;
    Some(
        LogLine {
            ip: cap["ip"].to_string(),
            method: cap["method"].to_string(),
            path: cap["path"].to_string(),
            status: cap["status"].parse().ok()?,
            size: cap["size"].parse().unwrap_or(0)
        }
    )
}



