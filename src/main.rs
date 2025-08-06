use clap::Parser;
use code_analyzer::{run_analysis, CliArgs};
use std::process;

fn main() {
    // Parse command line arguments
    let args = CliArgs::parse();

    // Run the analysis and handle results
    if let Err(error) = run_analysis(args) {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
