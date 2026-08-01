use regex::Regex;

pub mod summary;
pub mod status;
pub mod filter;
pub mod top;


pub struct LogLine<'a> {
    pub ip: &'a str,
    pub method: &'a str,
    pub path: &'a str,
    pub status: u16,
    pub size: u64
}

pub fn parse_line<'s>(line: &'s str, re: &Regex) -> Option<LogLine<'s>> {
    let cap = re.captures(line)?;
    Some(
        LogLine {
            ip: cap.name("ip")?.as_str(),
            method: cap.name("method")?.as_str(),
            path: cap.name("path")?.as_str(),
            status: cap["status"].parse().unwrap_or(0),
            size: cap["size"].parse().unwrap_or(0)
        }
    )
}



