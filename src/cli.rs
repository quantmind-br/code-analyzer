use clap::{Parser, ValueEnum};
use std::path::PathBuf;

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
    #[arg(long, value_enum, default_value_t = SortBy::Lines, help = "Sort output by specified metric")]
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

    /// Output JSON to custom file path
    #[arg(long, value_name = "FILE", help = "Custom path for JSON output file")]
    pub output_file: Option<PathBuf>,

    /// Show only top N results in terminal output
    #[arg(
        long,
        default_value_t = 50,
        help = "Limit terminal output to top N results"
    )]
    pub limit: usize,
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
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Terminal table output only
    Table,
    /// JSON output only
    Json,
    /// Both terminal table and JSON output
    Both,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Both => write!(f, "both"),
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
        matches!(self.output, OutputFormat::Json | OutputFormat::Both) || self.json_only
    }

    /// Check if terminal output should be displayed
    pub fn should_output_terminal(&self) -> bool {
        !self.json_only && matches!(self.output, OutputFormat::Table | OutputFormat::Both)
    }

    /// Get maximum file size in bytes
    pub fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size_mb as u64 * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_defaults() {
        let args = CliArgs::parse_from(&["code-analyzer"]);
        assert_eq!(args.min_lines, 1);
        assert!(matches!(args.sort, SortBy::Lines));
        assert!(matches!(args.output, OutputFormat::Table));
        assert!(!args.json_only);
        assert!(!args.verbose);
        assert_eq!(args.max_file_size_mb, 10);
        assert_eq!(args.limit, 50);
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
            sort: SortBy::Lines,
            output: OutputFormat::Table,
            json_only: false,
            verbose: false,
            languages: Vec::new(),
            exclude: Vec::new(),
            include_hidden: false,
            max_file_size_mb: 10,
            output_file: None,
            limit: 50,
        }
    }
}
