use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Analyse web-server access logs and report statistics
#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long, value_enum, default_value_t=OutputFormat::Table, global=true)]
    pub format: OutputFormat,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Summary {
        file_name: PathBuf,
    },
    Top {
        /// Path to logfile to analyse
        file_name: PathBuf,
        /// Number of top IPs to display
        #[arg(long, short, default_value_t = 10)]
        number: usize,
    },
    Status {
        file_name: PathBuf,
    },
    Filter {
        file_name: PathBuf,
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
