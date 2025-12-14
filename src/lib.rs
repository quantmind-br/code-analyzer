//! Code Analyzer Library
//!
//! A comprehensive library for analyzing codebases to identify refactoring candidates
//! using AST parsing across multiple programming languages.
//!
//! # Quick Start
//!
//! ```no_run
//! use code_analyzer::{run_analysis, CliArgs};
//!
//! let args = CliArgs {
//!     path: Some("./my-project".into()),
//!     verbose: true,
//!     ..Default::default()
//! };
//!
//! match run_analysis(args) {
//!     Ok(()) => println!("Analysis completed successfully"),
//!     Err(e) => eprintln!("Analysis failed: {}", e),
//! }
//! ```
//!
//! # Library Components
//!
//! - **Analyzer**: Core analysis engine with AST parsing and file discovery
//! - **Output**: Terminal and JSON output formatting
//! - **CLI**: Command-line interface definitions
//! - **Error**: Comprehensive error handling

use std::path::Path;

pub mod analyzer;
pub mod cli;
pub mod error;
pub mod output;

// Re-export main types for convenience
pub use analyzer::{
    analyze_project_simple, AnalysisReport, AnalyzerEngine, FileAnalysis, LanguageManager,
    ProjectSummary, SupportedLanguage,
};
pub use cli::{CliArgs, OutputFormat, SortBy};
pub use error::{AnalyzerError, Result};
pub use output::{
    display_analysis_results, export_analysis_json, generate_dual_output, JsonExporter,
    OutputManager, TerminalReporter,
};

/// Main entry point for running code analysis
///
/// This function orchestrates the complete analysis workflow:
/// 1. Validates CLI arguments
/// 2. Creates and configures the analyzer engine
/// 3. Runs the analysis on the target directory
/// 4. Generates output in the requested format(s)
///
/// # Arguments
///
/// * `args` - CLI arguments specifying analysis configuration
///
/// # Returns
///
/// * `Ok(())` if analysis completed successfully
/// * `Err(AnalyzerError)` if any step failed
///
/// # Examples
///
/// ```no_run
/// use code_analyzer::{run_analysis, CliArgs, OutputFormat};
///
/// let args = CliArgs {
///     path: Some("./src".into()),
///     output: OutputFormat::Both,
///     min_lines: 10,
///     verbose: true,
///     ..Default::default()
/// };
///
/// run_analysis(args).expect("Analysis failed");
/// ```
pub fn run_analysis(args: CliArgs) -> Result<()> {
    // Validate CLI arguments
    args.validate()?;

    if args.verbose {
        println!("Starting code analysis...");
        println!("Target: {}", args.target_path().display());

        if !args.languages.is_empty() {
            println!("Languages: {}", args.languages.join(", "));
        }

        println!("Min lines: {}", args.min_lines);
        if let Some(max_lines) = args.max_lines {
            println!("Max lines: {max_lines}");
        }

        println!("Output format: {}", args.output);
        println!();
    }

    // Create and configure analyzer engine
    let mut analyzer = AnalyzerEngine::from_cli_args(&args)?;

    // Run the analysis
    let report = analyzer.analyze_project(args.target_path(), &args)?;

    // Generate output based on compact mode or normal mode
    if args.compact {
        // Compact output: minimal table for CI/CD
        output::display_compact_table(&report.files, args.sort, args.limit);
    } else {
        // Normal output: full report with summary
        let output_manager = OutputManager::from_cli_args(&args);
        output_manager.generate_output(&report, &args)?;
    }

    if args.verbose {
        println!();
        println!("Analysis completed successfully!");
        println!("Files analyzed: {}", report.files.len());
        println!("Total lines: {}", report.summary.total_lines);
        println!("Total functions: {}", report.summary.total_functions);
        println!("Total classes: {}", report.summary.total_classes);
    }

    Ok(())
}

/// Run analysis with custom configuration
///
/// This function provides a more flexible interface for programmatic use,
/// allowing direct specification of analysis parameters without CLI argument parsing.
///
/// # Arguments
///
/// * `target_path` - Directory to analyze
/// * `config` - Analysis configuration
///
/// # Returns
///
/// * `Ok(AnalysisReport)` containing the analysis results
/// * `Err(AnalyzerError)` if analysis failed
pub fn run_analysis_with_config<P: AsRef<Path>>(
    target_path: P,
    config: AnalysisConfig,
) -> Result<AnalysisReport> {
    // Convert config to CLI args for compatibility
    let cli_args = CliArgs {
        path: Some(target_path.as_ref().to_path_buf()),
        languages: config.languages.clone(),
        min_lines: config.min_lines,
        max_lines: config.max_lines,
        include_hidden: config.include_hidden,
        max_file_size_mb: config.max_file_size_mb,
        verbose: config.verbose,
        ..Default::default()
    };

    // Create analyzer with proper language configuration
    let mut analyzer = AnalyzerEngine::from_cli_args(&cli_args)?;
    analyzer.analyze_project(target_path, &cli_args)
}

/// Simplified analysis configuration for programmatic use
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Languages to analyze (empty = all supported)
    pub languages: Vec<String>,

    /// Minimum lines of code to include
    pub min_lines: usize,

    /// Maximum lines of code to include (None = no limit)
    pub max_lines: Option<usize>,

    /// Include hidden files
    pub include_hidden: bool,

    /// Maximum file size in MB
    pub max_file_size_mb: usize,

    /// Show progress during analysis
    pub verbose: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            languages: Vec::new(),
            min_lines: 1,
            max_lines: None,
            include_hidden: false,
            max_file_size_mb: 10,
            verbose: false,
        }
    }
}

/// Quick analysis function for simple use cases
///
/// Analyzes a directory with default settings and returns the results.
/// Does not generate any output - just returns the analysis data.
///
/// # Arguments
///
/// * `target_path` - Directory to analyze
///
/// # Examples
///
/// ```no_run
/// use code_analyzer::analyze_directory;
///
/// match analyze_directory("./src") {
///     Ok(report) => {
///         println!("Found {} files", report.files.len());
///         println!("Total lines: {}", report.summary.total_lines);
///     }
///     Err(e) => eprintln!("Analysis failed: {}", e),
/// }
/// ```
pub fn analyze_directory<P: AsRef<Path>>(target_path: P) -> Result<AnalysisReport> {
    analyze_project_simple(target_path, None, None)
}

/// Quick analysis with language filtering
///
/// # Arguments
///
/// * `target_path` - Directory to analyze
/// * `languages` - Languages to include (e.g., ["rust", "python"])
///
/// # Examples
///
/// ```no_run
/// use code_analyzer::analyze_directory_filtered;
///
/// let report = analyze_directory_filtered("./src", vec!["rust".to_string()])?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn analyze_directory_filtered<P: AsRef<Path>>(
    target_path: P,
    languages: Vec<String>,
) -> Result<AnalysisReport> {
    analyze_project_simple(target_path, Some(languages), None)
}

/// Validate language list
///
/// Checks if all provided language names are supported.
///
/// # Arguments
///
/// * `languages` - List of language names to validate
///
/// # Returns
///
/// * `Ok(())` if all languages are supported
/// * `Err(AnalyzerError)` if any language is unsupported
pub fn validate_languages(languages: &[String]) -> Result<()> {
    analyzer::language::validate_language_list(languages)?;
    Ok(())
}

/// Get list of all supported languages
///
/// Returns a vector of all programming languages supported by the analyzer.
pub fn supported_languages() -> Vec<String> {
    SupportedLanguage::all_names()
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Check if a specific language is supported
///
/// # Arguments
///
/// * `language` - Language name to check
///
/// # Returns
///
/// * `true` if the language is supported
/// * `false` if the language is not supported
pub fn is_language_supported(language: &str) -> bool {
    language.parse::<SupportedLanguage>().is_ok()
}

/// Get file extension mappings for supported languages
///
/// Returns a mapping of file extensions to language names.
pub fn get_language_extensions() -> std::collections::HashMap<String, String> {
    let mut extensions = std::collections::HashMap::new();

    // Add mappings for each supported language
    extensions.insert("rs".to_string(), "rust".to_string());
    extensions.insert("js".to_string(), "javascript".to_string());
    extensions.insert("jsx".to_string(), "javascript".to_string());
    extensions.insert("ts".to_string(), "typescript".to_string());
    extensions.insert("tsx".to_string(), "typescript".to_string());
    extensions.insert("py".to_string(), "python".to_string());
    extensions.insert("java".to_string(), "java".to_string());
    extensions.insert("c".to_string(), "c".to_string());
    extensions.insert("h".to_string(), "c".to_string());
    extensions.insert("cpp".to_string(), "cpp".to_string());
    extensions.insert("cc".to_string(), "cpp".to_string());
    extensions.insert("cxx".to_string(), "cpp".to_string());
    extensions.insert("hpp".to_string(), "cpp".to_string());
    extensions.insert("go".to_string(), "go".to_string());

    extensions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        fs::write(
            root.join("main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n\nstruct Test {\n    value: i32,\n}",
        )
        .unwrap();

        fs::write(
            root.join("utils.py"),
            "def hello():\n    print('Hello from Python')\n\nclass Calculator:\n    pass",
        )
        .unwrap();

        dir
    }

    #[test]
    fn test_analyze_directory() {
        let test_dir = create_test_project();

        let result = analyze_directory(test_dir.path());
        if result.is_err() {
            eprintln!(
                "Analysis failed with error: {:?}",
                result.as_ref().unwrap_err()
            );
        }
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(!report.files.is_empty());
        assert!(report.summary.total_lines > 0);
        assert!(report.summary.total_functions > 0);
    }

    #[test]
    fn test_analyze_directory_filtered() {
        let test_dir = create_test_project();

        let result = analyze_directory_filtered(test_dir.path(), vec!["rust".to_string()]);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(!report.files.is_empty());
        // Should only contain Rust files
        assert!(report.files.iter().all(|f| f.language == "rust"));
    }

    #[test]
    fn test_run_analysis_with_config() {
        let test_dir = create_test_project();

        let config = AnalysisConfig {
            languages: vec!["python".to_string()],
            min_lines: 1,
            verbose: false,
            ..Default::default()
        };

        let result = run_analysis_with_config(test_dir.path(), config);
        assert!(result.is_ok());

        let report = result.unwrap();

        // Debug: Print all detected languages
        for file in &report.files {
            eprintln!(
                "Found file: {:?} with language: {}",
                file.path, file.language
            );
        }

        assert!(report.files.iter().all(|f| f.language == "python"));
    }

    #[test]
    fn test_supported_languages() {
        let languages = supported_languages();
        assert!(!languages.is_empty());
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
    }

    #[test]
    fn test_is_language_supported() {
        assert!(is_language_supported("rust"));
        assert!(is_language_supported("python"));
        assert!(is_language_supported("javascript"));
        assert!(!is_language_supported("unknown"));
    }

    #[test]
    fn test_validate_languages() {
        let valid_languages = vec!["rust".to_string(), "python".to_string()];
        assert!(validate_languages(&valid_languages).is_ok());

        let invalid_languages = vec!["rust".to_string(), "unknown".to_string()];
        assert!(validate_languages(&invalid_languages).is_err());
    }

    #[test]
    fn test_get_language_extensions() {
        let extensions = get_language_extensions();
        assert!(!extensions.is_empty());
        assert_eq!(extensions.get("rs"), Some(&"rust".to_string()));
        assert_eq!(extensions.get("py"), Some(&"python".to_string()));
        assert_eq!(extensions.get("js"), Some(&"javascript".to_string()));
    }

    #[test]
    fn test_analysis_config_default() {
        let config = AnalysisConfig::default();
        assert_eq!(config.min_lines, 1);
        assert_eq!(config.max_lines, None); // Check max_lines is None
        assert_eq!(config.max_file_size_mb, 10);
        assert!(!config.include_hidden);
        assert!(!config.verbose);
    }

    #[test]
    fn test_run_analysis_with_verbose_and_max_lines() {
        use std::path::PathBuf;

        let temp_dir = create_test_project();
        let args = CliArgs {
            path: Some(temp_dir.path().to_path_buf()),
            min_lines: 1,
            max_lines: Some(1000), // Set max_lines to trigger the uncovered line
            verbose: true,         // Set verbose to trigger the uncovered line
            ..Default::default()
        };

        let result = run_analysis(args);
        assert!(result.is_ok());
    }
}
