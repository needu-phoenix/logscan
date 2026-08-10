use regex::Regex;

pub mod filter;
pub mod status;
pub mod summary;
pub mod top;

pub const LOG_PATTERN: &str = r#"^(?P<ip>\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\s+(?P<identity>\S+)\s+(?P<user>\S+)\s+\[(?P<datetime>[^\]]*)\]\s+"(?P<method>\w+)\s+(?P<path>\S+)\s+(?P<protocol>[^"]*)"\s+(?P<status>\d{3})\s+(?P<size>\d+|-)"#;

#[derive(PartialEq, Debug)]
pub struct LogLine {
    pub ip: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub size: u64,
}

pub fn parse_line(line: &str, re: &Regex) -> Option<LogLine> {
    let cap = re.captures(line)?;
    Some(LogLine {
        ip: cap["ip"].to_string(),
        method: cap["method"].to_string(),
        path: cap["path"].to_string(),
        status: cap["status"].parse().ok()?,
        size: cap["size"].parse().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {

    use super::{LOG_PATTERN, LogLine, parse_line};
    use regex::Regex;

    fn get_regex() -> Regex {
        Regex::new(LOG_PATTERN).unwrap()
    }

    #[test]
    fn malformed_line_returns_none() {
        let re = get_regex();
        let line = "this is not a log line";
        let actual = parse_line(line, &re);

        assert_eq!(actual, None);
    }

    #[test]
    fn valid_line_parses_all_fields() {
        let line = r#"198.51.100.13 - - [10/Oct/2025:11:38:24 +0000] "GET /style.css HTTP/1.0" 200 128374 "https://example.com/" "curl/8.4.0""#;
        let re = get_regex();

        let actual = parse_line(line, &re);
        let expected = LogLine {
            ip: "198.51.100.13".to_string(),
            method: "GET".to_string(),
            path: "/style.css".to_string(),
            status: 200,
            size: 128374,
        };

        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn size_field_dash_parses_to_zero() {
        let line = r#"198.51.100.13 - - [10/Oct/2025:11:38:24 +0000] "GET /style.css HTTP/1.0" 200 - "https://example.com/" "curl/8.4.0""#;
        let re = get_regex();

        let actual = parse_line(line, &re);
        let expected = LogLine {
            ip: "198.51.100.13".to_string(),
            method: "GET".to_string(),
            path: "/style.css".to_string(),
            status: 200,
            size: 0,
        };

        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn empty_line_returns_none() {
        let line = r#""#;
        let re = get_regex();

        let actual = parse_line(line, &re);

        assert_eq!(actual, None);
    }

    #[test]
    fn missing_required_field_returns_none() {
        let line = r#"198.51.100.13 - - [10/Oct/2025:11:38:24 +0000] " /style.css HTTP/1.0" 200 - "https://example.com/" "curl/8.4.0""#;
        let re = get_regex();

        let actual = parse_line(line, &re);
        assert_eq!(actual, None);
    }

    #[test]
    fn ipv6_line_returns_none() {
        let line = r#"2001:db8::1 - - [10/Oct/2025:11:38:24 +0000] "GET /style.css HTTP/1.0" 200 - "https://example.com/" "curl/8.4.0""#;
        let re = get_regex();

        let actual = parse_line(line, &re);
        assert_eq!(actual, None);
    }
}
