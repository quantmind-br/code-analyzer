use std::path::Path;

use crate::analyzer::{AnalysisReport, FileAnalysis, ProjectSummary};
use crate::cli::{CliArgs, OutputFormat, SortBy};
use crate::error::{AnalyzerError, Result};

pub mod json;
pub mod terminal;

pub use json::{export_analysis_results, export_compact_json, JsonExporter};
pub use terminal::{apply_sorting, create_simple_table, display_compact_table, TerminalReporter};

/// Output manager that coordinates terminal and JSON output
pub struct OutputManager {
    terminal_reporter: TerminalReporter,
    json_exporter: JsonExporter,
    show_summary: bool,
}

impl OutputManager {
    /// Create a new output manager with default settings
    pub fn new() -> Self {
        Self {
            terminal_reporter: TerminalReporter::new(),
            json_exporter: JsonExporter::new(),
            show_summary: true,
        }
    }

    /// Create an output manager configured from CLI arguments
    pub fn from_cli_args(args: &CliArgs) -> Self {
        let mut terminal_reporter = TerminalReporter::new()
            .show_summary(!args.json_only)
            .color_enabled(args.should_use_colors());

        // Set base path for relative path display
        terminal_reporter = terminal_reporter.with_base_path(args.target_path());

        let json_exporter = JsonExporter::new().pretty_print(true); // Always use pretty print for files

        Self {
            terminal_reporter,
            json_exporter,
            show_summary: !args.json_only,
        }
    }

    /// Generate output according to CLI arguments
    pub fn generate_output(&self, report: &AnalysisReport, args: &CliArgs) -> Result<()> {
        // Determine what outputs to generate
        let show_terminal = args.should_output_terminal();
        let show_json = args.should_output_json();

        if !show_terminal && !show_json {
            return Err(AnalyzerError::validation_error(
                "No output format specified",
            ));
        }

        // Generate terminal output if requested
        if show_terminal {
            self.generate_terminal_output(report, args)?;
        }

        // Generate JSON output if requested
        if show_json {
            self.generate_json_output(report, args)?;

            if args.verbose {
                println!(
                    "JSON report saved to: {}",
                    args.json_output_path().display()
                );
            }
        }

        Ok(())
    }

    /// Generate terminal output
    pub fn generate_terminal_output(&self, report: &AnalysisReport, args: &CliArgs) -> Result<()> {
        self.terminal_reporter
            .display_report(report, args.sort, args.limit)?;

        Ok(())
    }

    /// Generate JSON output
    pub fn generate_json_output(&self, report: &AnalysisReport, args: &CliArgs) -> Result<()> {
        let output_path = args.json_output_path();

        // Always apply sorting and filtering based on CLI args
        // This ensures consistent behavior with terminal output
        self.json_exporter.export_filtered_report(
            report,
            &output_path,
            args.sort,
            Some(args.limit),
            if args.min_lines > 1 {
                Some(args.min_lines)
            } else {
                None
            },
            args.min_functions,
        )?;

        Ok(())
    }

    /// Display only file analysis table (no summary)
    pub fn display_files_only(
        &self,
        files: &[FileAnalysis],
        sort_by: SortBy,
        limit: usize,
    ) -> Result<()> {
        self.terminal_reporter
            .display_file_analysis_table(files, sort_by, limit)
    }

    /// Display only project summary
    pub fn display_summary_only(&self, summary: &ProjectSummary) -> Result<()> {
        self.terminal_reporter.display_project_summary(summary)
    }

    /// Export files to JSON without full report structure
    pub fn export_files_json_only<P: AsRef<Path>>(
        &self,
        files: &[FileAnalysis],
        path: P,
    ) -> Result<()> {
        self.json_exporter.export_files_only(files, path)
    }

    /// Configure terminal output options
    pub fn configure_terminal(&mut self, show_summary: bool, color_enabled: bool) -> &mut Self {
        self.terminal_reporter = self
            .terminal_reporter
            .clone_with_config(show_summary, color_enabled);
        self.show_summary = show_summary;
        self
    }

    /// Configure JSON output options
    pub fn configure_json(&mut self, pretty_print: bool) -> &mut Self {
        self.json_exporter = self.json_exporter.clone().pretty_print(pretty_print);
        self
    }

    /// Get a reference to the terminal reporter
    pub fn terminal_reporter(&self) -> &TerminalReporter {
        &self.terminal_reporter
    }

    /// Get a reference to the JSON exporter
    pub fn json_exporter(&self) -> &JsonExporter {
        &self.json_exporter
    }
}

impl Default for OutputManager {
    fn default() -> Self {
        Self::new()
    }
}

// Extension trait for TerminalReporter to support cloning with config
trait TerminalReporterExt {
    fn clone_with_config(&self, show_summary: bool, color_enabled: bool) -> TerminalReporter;
    fn base_path(&self) -> Option<std::path::PathBuf>;
}

impl TerminalReporterExt for TerminalReporter {
    fn clone_with_config(&self, show_summary: bool, color_enabled: bool) -> TerminalReporter {
        let mut reporter = TerminalReporter::new()
            .show_summary(show_summary)
            .color_enabled(color_enabled);
        if let Some(base) = self.base_path() {
            reporter = reporter.with_base_path(base);
        }
        reporter
    }

    fn base_path(&self) -> Option<std::path::PathBuf> {
        self.get_base_path().cloned()
    }
}

/// Convenience functions for quick output generation
///
/// Display analysis results to terminal with default formatting
pub fn display_analysis_results(
    report: &AnalysisReport,
    sort_by: SortBy,
    limit: usize,
) -> Result<()> {
    let manager = OutputManager::new();
    manager
        .terminal_reporter
        .display_report(report, sort_by, limit)
}

/// Export analysis results to JSON with default formatting
pub fn export_analysis_json<P: AsRef<Path>>(report: &AnalysisReport, path: P) -> Result<()> {
    let manager = OutputManager::new();
    manager.json_exporter.export_to_file(report, path)
}

/// Generate both terminal and JSON output
pub fn generate_dual_output<P: AsRef<Path>>(
    report: &AnalysisReport,
    json_path: P,
    sort_by: SortBy,
    limit: usize,
) -> Result<()> {
    let manager = OutputManager::new();

    // Display to terminal
    manager
        .terminal_reporter
        .display_report(report, sort_by, limit)?;

    // Export to JSON
    manager.json_exporter.export_to_file(report, json_path)?;

    Ok(())
}

/// Generate compact output for CI/CD integration
pub fn generate_compact_output<P: AsRef<Path>>(
    files: &[FileAnalysis],
    json_path: P,
    show_terminal: bool,
) -> Result<()> {
    if show_terminal {
        terminal::display_compact_results(files, SortBy::Lines);
    }

    export_compact_json(files, json_path)?;

    Ok(())
}

/// Output format detection and routing
pub fn route_output_by_format(
    report: &AnalysisReport,
    format: OutputFormat,
    json_path: Option<&Path>,
    sort_by: SortBy,
    limit: usize,
) -> Result<()> {
    let manager = OutputManager::new();

    match format {
        OutputFormat::Table => manager
            .terminal_reporter
            .display_report(report, sort_by, limit),
        OutputFormat::Json => {
            if let Some(path) = json_path {
                manager.json_exporter.export_to_file(report, path)
            } else {
                Err(AnalyzerError::validation_error(
                    "JSON output path not specified",
                ))
            }
        }
        OutputFormat::Both => {
            // Display to terminal
            manager
                .terminal_reporter
                .display_report(report, sort_by, limit)?;

            // Export to JSON
            if let Some(path) = json_path {
                manager.json_exporter.export_to_file(report, path)?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_test_report() -> AnalysisReport {
        let files = vec![
            crate::analyzer::FileAnalysis {
                path: PathBuf::from("src/main.rs"),
                language: "rust".to_string(),
                lines_of_code: 100,
                blank_lines: 10,
                comment_lines: 20,
                functions: 5,
                methods: 3,
                classes: 2,
                cyclomatic_complexity: 8,
                complexity_score: 3.2,
            },
            crate::analyzer::FileAnalysis {
                path: PathBuf::from("lib/utils.js"),
                language: "javascript".to_string(),
                lines_of_code: 75,
                blank_lines: 8,
                comment_lines: 15,
                functions: 3,
                methods: 2,
                classes: 1,
                cyclomatic_complexity: 5,
                complexity_score: 2.1,
            },
        ];

        let summary = crate::analyzer::create_project_summary(&files);

        let config = crate::analyzer::AnalysisConfig {
            target_path: PathBuf::from("./test"),
            languages: vec!["rust".to_string(), "javascript".to_string()],
            min_lines: 1,
            max_lines: None,
            include_hidden: false,
            max_file_size_mb: 10,
        };

        AnalysisReport {
            files,
            summary,
            config,
            generated_at: Utc::now(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn test_output_manager_creation() {
        let manager = OutputManager::new();
        assert!(manager.show_summary);

        let cli_args = CliArgs {
            json_only: true,
            ..Default::default()
        };

        let manager = OutputManager::from_cli_args(&cli_args);
        assert!(!manager.show_summary);
    }

    #[test]
    fn test_generate_output() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        let cli_args = CliArgs {
            output: OutputFormat::Both,
            output_file: Some(temp_file.path().to_path_buf()),
            verbose: false,
            ..Default::default()
        };

        let manager = OutputManager::from_cli_args(&cli_args);
        let result = manager.generate_output(&report, &cli_args);

        assert!(result.is_ok());

        // Verify JSON file was created
        assert!(temp_file.path().exists());
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_display_files_only() {
        let report = create_test_report();
        let manager = OutputManager::new();

        let result = manager.display_files_only(&report.files, SortBy::Lines, 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_files_json_only() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();
        let manager = OutputManager::new();

        let result = manager.export_files_json_only(&report.files, temp_file.path());
        assert!(result.is_ok());

        // Verify content
        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.starts_with('[')); // Should be an array
    }

    #[test]
    fn test_convenience_functions() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        // Test display_analysis_results (should not panic)
        let result = display_analysis_results(&report, SortBy::Lines, 5);
        assert!(result.is_ok());

        // Test export_analysis_json
        let result = export_analysis_json(&report, temp_file.path());
        assert!(result.is_ok());
        assert!(temp_file.path().exists());

        // Test generate_dual_output
        let temp_file2 = NamedTempFile::new().unwrap();
        let result = generate_dual_output(&report, temp_file2.path(), SortBy::Lines, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_route_output_by_format() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        // Test table format
        let result = route_output_by_format(&report, OutputFormat::Table, None, SortBy::Lines, 5);
        assert!(result.is_ok());

        // Test JSON format
        let result = route_output_by_format(
            &report,
            OutputFormat::Json,
            Some(temp_file.path()),
            SortBy::Lines,
            5,
        );
        assert!(result.is_ok());
        assert!(temp_file.path().exists());

        // Test both format
        let temp_file2 = NamedTempFile::new().unwrap();
        let result = route_output_by_format(
            &report,
            OutputFormat::Both,
            Some(temp_file2.path()),
            SortBy::Lines,
            5,
        );
        assert!(result.is_ok());
        assert!(temp_file2.path().exists());
    }

    #[test]
    fn test_generate_compact_output() {
        let report = create_test_report();
        let temp_file = NamedTempFile::new().unwrap();

        let result = generate_compact_output(&report.files, temp_file.path(), false);
        assert!(result.is_ok());
        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_manager_configuration() {
        let mut manager = OutputManager::new();

        manager
            .configure_terminal(false, false)
            .configure_json(false);

        assert!(!manager.show_summary);
    }
}
