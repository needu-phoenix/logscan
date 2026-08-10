use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Analyse web-server access logs and report statistics
#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long, short)]
    pub file_name: PathBuf,
    #[arg(long, value_enum, default_value_t=OutputFormat::Table, global=true)]
    pub format: OutputFormat,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Traffic statistics for the log
    Summary,
    /// Top IP addresses by request count
    Top {
        #[arg(long, short, default_value_t = 10)]
        number: usize,
    },
    /// Breakdown by status code
    Status,
    /// Show lines matching a given status code
    Filter {
        #[arg(short, long)]
        status: u16,
    },
}

/// Desired output format
#[derive(Debug, ValueEnum, Clone)]
pub enum OutputFormat {
    Table,
    Json,
}
