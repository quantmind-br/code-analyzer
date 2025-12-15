use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Color output mode
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum, Default)]
pub enum ColorMode {
    /// Auto-detect based on terminal (TTY)
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors (for piping)
    Never,
}

/// CLI arguments for the code analyzer application
#[derive(Parser)]
#[command(name = "code-analyzer")]
#[command(about = "Analyze codebases to identify refactoring candidates using AST parsing")]
#[command(version)]
#[command(
    long_about = "A powerful CLI tool that recursively analyzes directory trees, parsing source files with tree-sitter AST parsers, counting lines/functions/classes with language-specific accuracy, filtering files using .gitignore rules, and outputting both formatted terminal tables and structured JSON reports."
)]
pub struct CliArgs {
    /// Directory to analyze (default: current directory)
    #[arg(value_name = "PATH", help = "Path to the directory to analyze")]
    pub path: Option<PathBuf>,

    /// Minimum lines of code to include in results
    #[arg(
        long,
        default_value_t = 1,
        help = "Filter files with fewer than N lines"
    )]
    pub min_lines: usize,

    /// Maximum lines of code to include in results  
    #[arg(long, help = "Filter files with more than N lines")]
    pub max_lines: Option<usize>,

    /// Minimum number of functions to include in results
    #[arg(long, help = "Filter files with fewer than N functions")]
    pub min_functions: Option<usize>,

    /// Minimum number of classes to include in results
    #[arg(long, help = "Filter files with fewer than N classes")]
    pub min_classes: Option<usize>,

    /// Sort results by the specified criteria
    #[arg(long, value_enum, default_value_t = SortBy::Complexity, help = "Sort output by specified metric")]
    pub sort: SortBy,

    /// Output format selection
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, help = "Choose output format")]
    pub output: OutputFormat,

    /// Only output JSON file, skip terminal output
    #[arg(long, help = "Generate only JSON output file, no terminal display")]
    pub json_only: bool,

    /// Enable verbose output with progress reporting
    #[arg(short, long, help = "Show detailed progress and debug information")]
    pub verbose: bool,

    /// Languages to analyze (default: all supported languages)
    #[arg(
        long,
        value_delimiter = ',',
        help = "Comma-separated list of languages to analyze"
    )]
    pub languages: Vec<String>,

    /// Exclude file patterns (additional to .gitignore)
    #[arg(
        long,
        value_delimiter = ',',
        help = "Additional patterns to exclude from analysis"
    )]
    pub exclude: Vec<String>,

    /// Include hidden files in analysis
    #[arg(long, help = "Include hidden files and directories")]
    pub include_hidden: bool,

    /// Maximum file size to analyze (in MB)
    #[arg(
        long,
        default_value_t = 10,
        help = "Skip files larger than N megabytes"
    )]
    pub max_file_size_mb: usize,

    /// Compact output mode (minimal output for CI/CD pipelines)
    #[arg(
        long,
        help = "Compact output: essential metrics only (File, Language, Lines, CC, Score)"
    )]
    pub compact: bool,

    /// Output JSON to custom file path
    #[arg(long, value_name = "FILE", help = "Custom path for JSON output file")]
    pub output_file: Option<PathBuf>,

    /// Show only top N results in terminal output
    #[arg(
        long,
        default_value_t = 10,
        help = "Limit terminal output to top N results"
    )]
    pub limit: usize,

    /// Color output mode
    #[arg(
        long,
        value_enum,
        default_value_t = ColorMode::Auto,
        help = "Control color output (auto, always, never)"
    )]
    pub color: ColorMode,

    // === Phase 1: Custom Refactoring Thresholds ===
    /// Maximum complexity score threshold for refactoring candidates
    #[arg(
        long,
        value_name = "SCORE",
        help = "Complexity score threshold for refactoring candidates (default: 10.0)"
    )]
    pub max_complexity_score: Option<f64>,

    /// Maximum cyclomatic complexity threshold
    #[arg(
        long,
        value_name = "CC",
        help = "Cyclomatic complexity threshold per NIST guidance (default: 15)"
    )]
    pub max_cc: Option<usize>,

    /// Maximum lines of code threshold for refactoring candidates
    #[arg(
        long,
        value_name = "LINES",
        help = "Lines of code threshold for large file detection (default: 500)"
    )]
    pub max_loc: Option<usize>,

    /// Maximum functions per file threshold
    #[arg(
        long,
        value_name = "COUNT",
        help = "Function count threshold for refactoring candidates (default: 25)"
    )]
    pub max_functions_per_file: Option<usize>,

    // === Phase 2: Git Integration ===
    /// Only analyze files changed since the specified git commit
    #[arg(
        long,
        value_name = "COMMIT",
        help = "Only analyze files changed since this commit (e.g., HEAD~1, main, abc123)"
    )]
    pub only_changed_since: Option<String>,

    // === Phase 4: CI Mode ===
    /// CI mode: exit with code 2 if refactoring candidates found
    #[arg(
        long,
        help = "CI mode: exit code 2 if refactoring candidates exceed threshold"
    )]
    pub ci: bool,

    /// Maximum allowed candidates in CI mode before failing
    #[arg(
        long,
        default_value_t = 0,
        help = "Max allowed refactoring candidates in CI mode (0 = any triggers exit code 2)"
    )]
    pub ci_max_candidates: usize,
}

/// Sorting criteria for analysis results
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
pub enum SortBy {
    /// Sort by lines of code (descending)
    Lines,
    /// Sort by number of functions (descending)
    Functions,
    /// Sort by number of classes (descending)
    Classes,
    /// Sort by file name (ascending)
    Name,
    /// Sort by file path (ascending)
    Path,
    /// Sort by complexity score (descending)
    Complexity,
}

impl std::fmt::Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortBy::Lines => write!(f, "lines"),
            SortBy::Functions => write!(f, "functions"),
            SortBy::Classes => write!(f, "classes"),
            SortBy::Name => write!(f, "name"),
            SortBy::Path => write!(f, "path"),
            SortBy::Complexity => write!(f, "complexity"),
        }
    }
}

/// Output format options
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Terminal table output only
    Table,
    /// JSON output only (full report)
    Json,
    /// Both terminal table and JSON output
    Both,
    /// JSON with only file analysis data (no summary/config)
    #[value(name = "json-files-only")]
    JsonFilesOnly,
    /// JSON with only project summary (no individual files)
    #[value(name = "json-summary-only")]
    JsonSummaryOnly,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Both => write!(f, "both"),
            OutputFormat::JsonFilesOnly => write!(f, "json-files-only"),
            OutputFormat::JsonSummaryOnly => write!(f, "json-summary-only"),
        }
    }
}

impl CliArgs {
    /// Validate CLI arguments and return meaningful errors
    pub fn validate(&self) -> Result<(), crate::error::AnalyzerError> {
        // Validate path if provided
        if let Some(ref path) = self.path {
            if !path.exists() {
                return Err(crate::error::AnalyzerError::invalid_path(path));
            }
            if !path.is_dir() {
                return Err(crate::error::AnalyzerError::validation_error(format!(
                    "Path must be a directory: {}",
                    path.display()
                )));
            }
        }

        // Validate min/max lines constraints
        if let Some(max_lines) = self.max_lines {
            if self.min_lines >= max_lines {
                return Err(crate::error::AnalyzerError::validation_error(
                    "min-lines must be less than max-lines",
                ));
            }
        }

        // Validate file size limit
        if self.max_file_size_mb == 0 {
            return Err(crate::error::AnalyzerError::validation_error(
                "max-file-size-mb must be greater than 0",
            ));
        }

        // Validate limit
        if self.limit == 0 {
            return Err(crate::error::AnalyzerError::validation_error(
                "limit must be greater than 0",
            ));
        }

        // Validate output file path if provided
        if let Some(ref output_path) = self.output_file {
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    return Err(crate::error::AnalyzerError::validation_error(format!(
                        "Output directory does not exist: {}",
                        parent.display()
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get the target path for analysis (current directory if not specified)
    pub fn target_path(&self) -> PathBuf {
        self.path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Get the output file path (default to refactor-candidates.json)
    pub fn json_output_path(&self) -> PathBuf {
        self.output_file
            .as_ref()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("refactor-candidates.json"))
    }

    /// Check if JSON output should be generated
    pub fn should_output_json(&self) -> bool {
        matches!(
            self.output,
            OutputFormat::Json
                | OutputFormat::Both
                | OutputFormat::JsonFilesOnly
                | OutputFormat::JsonSummaryOnly
        ) || self.json_only
    }

    /// Check if terminal output should be displayed
    pub fn should_output_terminal(&self) -> bool {
        !self.json_only && matches!(self.output, OutputFormat::Table | OutputFormat::Both)
    }

    /// Get maximum file size in bytes
    pub fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size_mb as u64 * 1024 * 1024
    }

    /// Determine if colors should be used based on ColorMode and TTY detection
    pub fn should_use_colors(&self) -> bool {
        match self.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_defaults() {
        let args = CliArgs::parse_from(&["code-analyzer"]);
        assert_eq!(args.min_lines, 1);
        assert!(matches!(args.sort, SortBy::Complexity));
        assert!(matches!(args.output, OutputFormat::Table));
        assert!(!args.json_only);
        assert!(!args.verbose);
        assert_eq!(args.max_file_size_mb, 10);
        assert_eq!(args.limit, 10);
    }

    #[test]
    fn test_sort_by_display() {
        assert_eq!(SortBy::Lines.to_string(), "lines");
        assert_eq!(SortBy::Functions.to_string(), "functions");
        assert_eq!(SortBy::Classes.to_string(), "classes");
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Table.to_string(), "table");
        assert_eq!(OutputFormat::Json.to_string(), "json");
        assert_eq!(OutputFormat::Both.to_string(), "both");
    }

    #[test]
    fn test_target_path() {
        let args = CliArgs {
            path: Some(PathBuf::from("/test/path")),
            ..Default::default()
        };
        assert_eq!(args.target_path(), PathBuf::from("/test/path"));

        let args = CliArgs {
            path: None,
            ..Default::default()
        };
        // Should return current directory or "." fallback
        assert!(!args.target_path().as_os_str().is_empty());
    }

    #[test]
    fn test_json_output_path() {
        let args = CliArgs {
            output_file: Some(PathBuf::from("custom.json")),
            ..Default::default()
        };
        assert_eq!(args.json_output_path(), PathBuf::from("custom.json"));

        let args = CliArgs {
            output_file: None,
            ..Default::default()
        };
        assert_eq!(
            args.json_output_path(),
            PathBuf::from("refactor-candidates.json")
        );
    }

    #[test]
    fn test_output_logic() {
        let args = CliArgs {
            output: OutputFormat::Table,
            json_only: false,
            ..Default::default()
        };
        assert!(args.should_output_terminal());
        assert!(!args.should_output_json());

        let args = CliArgs {
            output: OutputFormat::Json,
            json_only: false,
            ..Default::default()
        };
        assert!(!args.should_output_terminal());
        assert!(args.should_output_json());

        let args = CliArgs {
            output: OutputFormat::Both,
            json_only: false,
            ..Default::default()
        };
        assert!(args.should_output_terminal());
        assert!(args.should_output_json());

        let args = CliArgs {
            output: OutputFormat::Table,
            json_only: true,
            ..Default::default()
        };
        assert!(!args.should_output_terminal());
        assert!(args.should_output_json());
    }

    #[test]
    fn test_validate_valid_args() {
        let args = CliArgs {
            path: Some(PathBuf::from(".")), // Current directory exists
            min_lines: 1,
            max_lines: Some(1000),
            max_file_size_mb: 10,
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_nonexistent_path() {
        let args = CliArgs {
            path: Some(PathBuf::from("/nonexistent/path")),
            ..Default::default()
        };
        let result = args.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid path"));
    }

    #[test]
    fn test_validate_file_instead_of_directory() {
        // Create a temporary file
        let temp_file = std::env::temp_dir().join("test_file.txt");
        std::fs::write(&temp_file, "test").unwrap();

        let args = CliArgs {
            path: Some(temp_file.clone()),
            ..Default::default()
        };
        let result = args.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be a directory"));

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_validate_min_lines_greater_than_max() {
        let args = CliArgs {
            min_lines: 100,
            max_lines: Some(50),
            ..Default::default()
        };
        let result = args.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("min-lines must be less than max-lines"));
    }

    #[test]
    fn test_validate_max_lines_equal_to_min_lines() {
        let args = CliArgs {
            min_lines: 50,
            max_lines: Some(50),
            ..Default::default()
        };
        let result = args.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("min-lines must be less than max-lines"));
    }

    #[test]
    fn test_validate_zero_file_size() {
        let args = CliArgs {
            max_file_size_mb: 0,
            ..Default::default()
        };
        let result = args.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("max-file-size-mb must be greater than 0"));
    }

    #[test]
    fn test_validate_with_none_path() {
        // None path should be valid (defaults to current directory)
        let args = CliArgs {
            path: None,
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_with_only_min_lines() {
        // No max_lines means no constraint
        let args = CliArgs {
            min_lines: 10,
            max_lines: None,
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }
}

// Implement Default for testing convenience
impl Default for CliArgs {
    fn default() -> Self {
        Self {
            path: None,
            min_lines: 1,
            max_lines: None,
            min_functions: None,
            min_classes: None,
            sort: SortBy::Complexity,
            output: OutputFormat::Table,
            json_only: false,
            verbose: false,
            languages: Vec::new(),
            exclude: Vec::new(),
            include_hidden: false,
            max_file_size_mb: 10,
            compact: false,
            output_file: None,
            limit: 10,
            color: ColorMode::Auto,
            // Phase 1: Custom thresholds (None = use defaults)
            max_complexity_score: None,
            max_cc: None,
            max_loc: None,
            max_functions_per_file: None,
            // Phase 2: Git integration
            only_changed_since: None,
            // Phase 4: CI mode
            ci: false,
            ci_max_candidates: 0,
        }
    }
}
