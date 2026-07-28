use std::{io, path::PathBuf};

pub enum ScanError {
    Io { path: PathBuf, source: io::Error },
    Parse { line_no: usize },
}
