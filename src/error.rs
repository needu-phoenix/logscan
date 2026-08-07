use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("failed to read '{path}': {source}")]
    Io {
        path: PathBuf, 
        source: io::Error
    },

    #[error("error reading input: {0}")]
    Read(#[from] io::Error),

    #[error("failed to serialize output: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("invalid regex pattern: {0}")]
    Regex(#[from] regex::Error)
}