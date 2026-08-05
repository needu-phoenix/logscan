use clap::Parser;
use logscan::{cli::Cli, run};

fn main() {
    let args = Cli::parse();
    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
