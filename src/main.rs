use clap::Parser;
use code_analyzer::{
    identify_refactoring_candidates, run_analysis, run_analysis_returning_report, CliArgs,
    RefactoringThresholds,
};
use std::process;

// Exit codes for CI integration
const EXIT_SUCCESS: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_CANDIDATES_EXCEEDED: i32 = 2;

fn main() {
    // Parse command line arguments
    let args = CliArgs::parse();

    // Check if CI mode is enabled
    if args.ci {
        run_ci_mode(args);
    } else {
        // Normal mode
        if let Err(error) = run_analysis(args) {
            eprintln!("Error: {error}");
            process::exit(EXIT_ERROR);
        }
    }
}

/// Run in CI mode with exit codes based on refactoring candidates
fn run_ci_mode(args: CliArgs) {
    // Store CI settings before running analysis
    let ci_max = args.ci_max_candidates;
    let thresholds = RefactoringThresholds::from_cli(&args);

    match run_analysis_returning_report(args) {
        Ok(report) => {
            // Identify refactoring candidates using configured thresholds
            let candidates = identify_refactoring_candidates(&report.files, &thresholds);

            if candidates.len() > ci_max {
                eprintln!();
                eprintln!(
                    "CI check failed: {} refactoring candidates found (max allowed: {})",
                    candidates.len(),
                    ci_max
                );
                eprintln!("Candidates:");
                for candidate in candidates.iter().take(10) {
                    eprintln!(
                        "  - {} (Score: {:.2}, CC: {}) - {}",
                        candidate.file.path.display(),
                        candidate.file.complexity_score,
                        candidate.file.cyclomatic_complexity,
                        candidate.reasons_string()
                    );
                }
                if candidates.len() > 10 {
                    eprintln!("  ... and {} more", candidates.len() - 10);
                }
                process::exit(EXIT_CANDIDATES_EXCEEDED);
            } else {
                if candidates.is_empty() {
                    eprintln!("CI check passed: No refactoring candidates found");
                } else {
                    eprintln!(
                        "CI check passed: {} refactoring candidates found (within limit of {})",
                        candidates.len(),
                        ci_max
                    );
                }
                process::exit(EXIT_SUCCESS);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(EXIT_ERROR);
        }
    }
}
