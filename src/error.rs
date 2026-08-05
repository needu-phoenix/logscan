use std::{error, fmt, io, path::PathBuf};

#[derive(Debug)]
pub enum ScanError {
    Io { path: PathBuf, source: io::Error},
    Read(io::Error),
    Serialize(serde_json::Error),
    Regex(regex::Error)
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScanError::Io { path, source } => {
                write!(f, "failed to read '{}': {}", path.display(), source)
            }
            ScanError::Read(e) => write!(f, "error reading input: {e}"),
            ScanError::Serialize(e) => write!(f, "failed to serialize output: {e}"),
            ScanError::Regex(e) => write!(f, "invalid regex pattern: {e}")
        }
    }
}

impl error::Error for ScanError {}

impl From<serde_json::Error> for ScanError {
    fn from(e: serde_json::Error) -> Self {
        ScanError::Serialize(e)
    }
}

impl From<regex::Error> for ScanError {
    fn from(e: regex::Error) -> Self {
        ScanError::Regex(e)
    }
}

impl From<io::Error> for ScanError {
    fn from(e: io::Error) -> Self {
        ScanError::Read(e)
    }
}