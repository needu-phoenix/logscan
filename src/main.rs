use clap::Parser;
use logscan::cli::Cli;
fn main() {
    let args = Cli::parse();
    println!("Args: {:?}", args);
}
