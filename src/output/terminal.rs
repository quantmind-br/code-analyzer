use crate::analyzer::parser::{AnalysisReport, FileAnalysis, ProjectSummary};
use crate::cli::SortBy;
use crate::error::{ParseWarning, Result};
use prettytable::{format, row, Cell, Row, Table};
use std::cmp::Ordering;

/// Terminal reporter for displaying analysis results as formatted tables
pub struct TerminalReporter {
    show_summary: bool,
    color_enabled: bool,
}

impl TerminalReporter {
    /// Create a new terminal reporter with default settings
    pub fn new() -> Self {
        Self {
            show_summary: true,
            color_enabled: true,
        }
    }

    /// Enable or disable summary display
    pub fn show_summary(mut self, show: bool) -> Self {
        self.show_summary = show;
        self
    }

    /// Enable or disable colored output
    pub fn color_enabled(mut self, enabled: bool) -> Self {
        self.color_enabled = enabled;
        self
    }

    /// Display the complete analysis report
    pub fn display_report(
        &self,
        report: &AnalysisReport,
        sort_by: SortBy,
        limit: usize,
    ) -> Result<()> {
        println!("Code Analysis Report");
        println!("===================");
        println!();

        if self.show_summary {
            self.display_project_summary(&report.summary)?;
            println!();
        }

        self.display_file_analysis_table(&report.files, sort_by, limit)?;

        if report.files.len() > limit {
            println!();
            println!(
                "Showing top {} results. Total files analyzed: {}",
                limit,
                report.files.len()
            );
        }

        // Display warnings if any
        if !report.warnings.is_empty() {
            println!();
            self.display_warnings(&report.warnings)?;
        }

        Ok(())
    }

    /// Display parse warnings
    pub fn display_warnings(&self, warnings: &[ParseWarning]) -> Result<()> {
        if warnings.is_empty() {
            return Ok(());
        }

        println!("Warnings ({}):", warnings.len());
        println!("─────────────");

        for warning in warnings {
            let warning_type = match warning.warning_type {
                crate::error::WarningType::SyntaxError => "⚠ Syntax",
                crate::error::WarningType::PartialParse => "⚠ Partial",
                crate::error::WarningType::EncodingError => "⚠ Encoding",
            };

            println!(
                "{}: {} - {}",
                warning_type,
                warning.file_path.display(),
                warning.message
            );
        }

        Ok(())
    }

    /// Display the main file analysis table
    pub fn display_file_analysis_table(
        &self,
        files: &[FileAnalysis],
        sort_by: SortBy,
        limit: usize,
    ) -> Result<()> {
        if files.is_empty() {
            println!("No files found matching the criteria.");
            return Ok(());
        }

        let table = self.format_analysis_table(files, sort_by, limit)?;
        table.printstd();

        Ok(())
    }

    /// Format the analysis results as a table
    pub fn format_analysis_table(
        &self,
        files: &[FileAnalysis],
        sort_by: SortBy,
        limit: usize,
    ) -> Result<Table> {
        let mut table = Table::new();

        // Set table format
        if self.color_enabled {
            table.set_format(*format::consts::FORMAT_DEFAULT);
        } else {
            table.set_format(*format::consts::FORMAT_NO_COLSEP);
        }

        // Add headers
        table.add_row(row![
            bFg->"File",
            bFg->"Language",
            bFg->"Lines",
            bFg->"Blank",
            bFg->"Comments",
            bFg->"Funcs",
            bFg->"Methods",
            bFg->"Classes",
            bFg->"CC",
            bFg->"Score"
        ]);

        // Sort files according to the specified criteria
        let mut sorted_files = files.to_vec();
        apply_sorting(&mut sorted_files, sort_by);

        // Add rows for each file (limited by the specified limit)
        for file in sorted_files.iter().take(limit) {
            let path_display = self.format_file_path(&file.path);

            // Color-code cyclomatic complexity
            let cc_cell = if self.color_enabled {
                self.format_cyclomatic_cell(file.cyclomatic_complexity)
            } else {
                Cell::new(&file.cyclomatic_complexity.to_string())
            };

            // Color-code complexity scores
            let score_cell = if self.color_enabled {
                self.format_complexity_cell(file.complexity_score)
            } else {
                Cell::new(&format!("{:.2}", file.complexity_score))
            };

            table.add_row(Row::new(vec![
                Cell::new(&path_display),
                Cell::new(&file.language),
                Cell::new(&file.lines_of_code.to_string()).style_spec("r"),
                Cell::new(&file.blank_lines.to_string()).style_spec("r"),
                Cell::new(&file.comment_lines.to_string()).style_spec("r"),
                Cell::new(&file.functions.to_string()).style_spec("r"),
                Cell::new(&file.methods.to_string()).style_spec("r"),
                Cell::new(&file.classes.to_string()).style_spec("r"),
                cc_cell.style_spec("r"),
                score_cell.style_spec("r"),
            ]));
        }

        Ok(table)
    }

    /// Display project summary statistics
    pub fn display_project_summary(&self, summary: &ProjectSummary) -> Result<()> {
        println!("Project Summary:");
        println!("├─ Files analyzed: {}", summary.total_files);
        println!("├─ Total lines: {}", summary.total_lines);
        println!("├─ Functions: {}", summary.total_functions);
        println!("├─ Methods: {}", summary.total_methods);
        println!("└─ Classes: {}", summary.total_classes);

        if !summary.language_breakdown.is_empty() {
            println!();
            self.display_language_breakdown(&summary.language_breakdown)?;
        }

        Ok(())
    }

    /// Display language breakdown statistics
    fn display_language_breakdown(
        &self,
        breakdown: &std::collections::HashMap<String, crate::analyzer::parser::LanguageStats>,
    ) -> Result<()> {
        println!("Language Breakdown:");

        let mut langs: Vec<_> = breakdown.iter().collect();
        langs.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.total_lines));

        for (i, (lang, stats)) in langs.iter().enumerate() {
            let prefix = if i == langs.len() - 1 {
                "└─"
            } else {
                "├─"
            };
            println!(
                "{} {}: {} files, {} lines (avg {:.1} functions, {:.1} classes per file)",
                prefix,
                lang,
                stats.file_count,
                stats.total_lines,
                stats.avg_functions_per_file,
                stats.avg_classes_per_file
            );
        }

        Ok(())
    }

    /// Display top files by different criteria
    pub fn display_top_files(&self, summary: &ProjectSummary) -> Result<()> {
        if !summary.largest_files.is_empty() {
            println!("\nTop Files by Size:");
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_CLEAN);
            table.add_row(row![bFg->"Rank", bFg->"File", bFg->"Lines", bFg->"Language"]);

            for (i, file) in summary.largest_files.iter().enumerate().take(5) {
                table.add_row(row![
                    (i + 1).to_string(),
                    self.format_file_path(&file.path),
                    file.total_lines().to_string(),
                    file.language
                ]);
            }
            table.printstd();
        }

        if !summary.most_complex_files.is_empty() {
            println!("\nMost Complex Files:");
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_CLEAN);
            table.add_row(
                row![bFg->"Rank", bFg->"File", bFg->"Functions", bFg->"Classes", bFg->"CC", bFg->"Score"],
            );

            for (i, file) in summary.most_complex_files.iter().enumerate().take(5) {
                table.add_row(row![
                    (i + 1).to_string(),
                    self.format_file_path(&file.path),
                    file.functions.to_string(),
                    file.classes.to_string(),
                    file.cyclomatic_complexity.to_string(),
                    format!("{:.2}", file.complexity_score)
                ]);
            }
            table.printstd();
        }

        Ok(())
    }

    /// Format file path for display (truncate if too long)
    fn format_file_path(&self, path: &std::path::Path) -> String {
        let path_str = path.display().to_string();
        const MAX_PATH_LENGTH: usize = 60;

        if path_str.len() > MAX_PATH_LENGTH {
            let start = path_str.len() - MAX_PATH_LENGTH + 3;
            format!("...{}", &path_str[start..])
        } else {
            path_str
        }
    }

    /// Format complexity cell with color coding
    fn format_complexity_cell(&self, complexity: f64) -> Cell {
        let complexity_str = format!("{complexity:.2}");

        if complexity < 2.0 {
            Cell::new(&complexity_str).style_spec("Fg")
        } else if complexity < 5.0 {
            Cell::new(&complexity_str).style_spec("Fy")
        } else {
            Cell::new(&complexity_str).style_spec("Fr")
        }
    }

    /// Format cyclomatic complexity cell with color coding
    /// CC of 1-10: low (green), 11-20: moderate (yellow), 21+: high (red)
    fn format_cyclomatic_cell(&self, cc: usize) -> Cell {
        let cc_str = cc.to_string();

        if cc <= 10 {
            Cell::new(&cc_str).style_spec("Fg")
        } else if cc <= 20 {
            Cell::new(&cc_str).style_spec("Fy")
        } else {
            Cell::new(&cc_str).style_spec("Fr")
        }
    }
}

impl Default for TerminalReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply sorting to a vector of file analyses
pub fn apply_sorting(files: &mut [FileAnalysis], sort_by: SortBy) {
    match sort_by {
        SortBy::Lines => {
            files.sort_by_key(|f| std::cmp::Reverse(f.lines_of_code));
        }
        SortBy::Functions => {
            files.sort_by_key(|f| std::cmp::Reverse(f.functions));
        }
        SortBy::Classes => {
            files.sort_by_key(|f| std::cmp::Reverse(f.classes));
        }
        SortBy::Name => {
            files.sort_by(|a, b| {
                a.path
                    .file_name()
                    .unwrap_or_default()
                    .cmp(b.path.file_name().unwrap_or_default())
            });
        }
        SortBy::Path => {
            files.sort_by(|a, b| a.path.cmp(&b.path));
        }
        SortBy::Complexity => {
            files.sort_by(|a, b| {
                b.complexity_score
                    .partial_cmp(&a.complexity_score)
                    .unwrap_or(Ordering::Equal)
            });
        }
    }
}

/// Helper function to create a simple table with analysis results
pub fn create_simple_table(files: &[FileAnalysis], limit: usize) -> Table {
    let reporter = TerminalReporter::new();
    reporter
        .format_analysis_table(files, SortBy::Lines, limit)
        .unwrap_or_else(|_| {
            let mut table = Table::new();
            table.add_row(row!["Error creating table"]);
            table
        })
}

/// Display analysis results to stdout in a compact format
pub fn display_compact_results(files: &[FileAnalysis], sort_by: SortBy) {
    if files.is_empty() {
        println!("No files found.");
        return;
    }

    let reporter = TerminalReporter::new()
        .show_summary(false)
        .color_enabled(false);

    if let Err(e) = reporter.display_file_analysis_table(files, sort_by, 20) {
        eprintln!("Error displaying results: {e}");
    }
}

/// Display compact table with only essential metrics (for CI/CD)
pub fn display_compact_table(files: &[FileAnalysis], sort_by: SortBy, limit: usize) {
    if files.is_empty() {
        println!("No files found.");
        return;
    }

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);

    // Compact headers: only essential metrics
    table.add_row(row![bFg->"File", bFg->"Lang", bFg->"Lines", bFg->"CC", bFg->"Score"]);

    // Sort files
    let mut sorted_files = files.to_vec();
    apply_sorting(&mut sorted_files, sort_by);

    // Add rows
    for file in sorted_files.iter().take(limit) {
        let path_str = file.path.display().to_string();
        let short_path = if path_str.len() > 50 {
            format!("...{}", &path_str[path_str.len() - 47..])
        } else {
            path_str
        };

        table.add_row(row![
            short_path,
            file.language,
            file.lines_of_code.to_string(),
            file.cyclomatic_complexity.to_string(),
            format!("{:.1}", file.complexity_score)
        ]);
    }

    table.printstd();

    if files.len() > limit {
        println!("(showing {} of {} files)", limit, files.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_file_analysis() -> Vec<FileAnalysis> {
        vec![
            FileAnalysis {
                path: PathBuf::from("src/main.rs"),
                language: "rust".to_string(),
                lines_of_code: 150,
                blank_lines: 20,
                comment_lines: 30,
                functions: 8,
                methods: 4,
                classes: 2,
                cyclomatic_complexity: 12,
                complexity_score: 3.5,
            },
            FileAnalysis {
                path: PathBuf::from("lib/utils.js"),
                language: "javascript".to_string(),
                lines_of_code: 75,
                blank_lines: 10,
                comment_lines: 15,
                functions: 5,
                methods: 3,
                classes: 1,
                cyclomatic_complexity: 6,
                complexity_score: 2.1,
            },
            FileAnalysis {
                path: PathBuf::from("tests/test_module.py"),
                language: "python".to_string(),
                lines_of_code: 200,
                blank_lines: 25,
                comment_lines: 40,
                functions: 12,
                methods: 8,
                classes: 3,
                cyclomatic_complexity: 18,
                complexity_score: 4.8,
            },
        ]
    }

    #[test]
    fn test_apply_sorting() {
        let mut files = create_test_file_analysis();

        // Test sorting by lines
        apply_sorting(&mut files, SortBy::Lines);
        assert_eq!(files[0].lines_of_code, 200); // Python file should be first

        // Test sorting by functions
        apply_sorting(&mut files, SortBy::Functions);
        assert_eq!(files[0].functions, 12); // Python file should be first

        // Test sorting by name
        apply_sorting(&mut files, SortBy::Name);
        // Should be sorted alphabetically by filename
        assert!(files[0].path.file_name().unwrap() <= files[1].path.file_name().unwrap());
    }

    #[test]
    fn test_terminal_reporter_creation() {
        let reporter = TerminalReporter::new();
        assert!(reporter.show_summary);
        assert!(reporter.color_enabled);

        let reporter = TerminalReporter::new()
            .show_summary(false)
            .color_enabled(false);
        assert!(!reporter.show_summary);
        assert!(!reporter.color_enabled);
    }

    #[test]
    fn test_format_analysis_table() {
        let files = create_test_file_analysis();
        let reporter = TerminalReporter::new();

        let table = reporter
            .format_analysis_table(&files, SortBy::Lines, 10)
            .unwrap();

        // Table should have header row plus data rows
        assert!(table.len() > files.len()); // Including header
    }

    #[test]
    fn test_format_file_path() {
        let reporter = TerminalReporter::new();

        // Short path should remain unchanged
        let short_path = PathBuf::from("src/main.rs");
        assert_eq!(reporter.format_file_path(&short_path), "src/main.rs");

        // Long path should be truncated
        let long_path =
            PathBuf::from("very/long/path/to/some/deeply/nested/directory/structure/file.rs");
        let formatted = reporter.format_file_path(&long_path);
        assert!(formatted.starts_with("..."));
        assert!(formatted.len() <= 63); // 60 + "..."
    }

    #[test]
    fn test_create_simple_table() {
        let files = create_test_file_analysis();
        let table = create_simple_table(&files, 5);

        // Should have at least one row (header)
        assert!(table.len() > 0);
    }

    #[test]
    fn test_display_compact_results() {
        let files = create_test_file_analysis();

        // This should not panic
        display_compact_results(&files, SortBy::Lines);

        // Test with empty files
        let empty_files = vec![];
        display_compact_results(&empty_files, SortBy::Lines);
    }

    #[test]
    fn test_display_warnings() {
        let reporter = TerminalReporter::new();

        // Test with empty warnings (should not panic)
        let warnings = vec![];
        let result = reporter.display_warnings(&warnings);
        assert!(result.is_ok());

        // Test with syntax error warning
        let warnings = vec![ParseWarning {
            file_path: PathBuf::from("test.rs"),
            warning_type: crate::error::WarningType::SyntaxError,
            message: "Parse error".to_string(),
        }];
        let result = reporter.display_warnings(&warnings);
        assert!(result.is_ok());

        // Test with encoding error warning
        let warnings = vec![ParseWarning {
            file_path: PathBuf::from("test.py"),
            warning_type: crate::error::WarningType::EncodingError,
            message: "Invalid encoding".to_string(),
        }];
        let result = reporter.display_warnings(&warnings);
        assert!(result.is_ok());

        // Test with partial parse warning
        let warnings = vec![ParseWarning {
            file_path: PathBuf::from("test.js"),
            warning_type: crate::error::WarningType::PartialParse,
            message: "Incomplete parse".to_string(),
        }];
        let result = reporter.display_warnings(&warnings);
        assert!(result.is_ok());

        // Test with multiple warnings
        let warnings = vec![
            ParseWarning {
                file_path: PathBuf::from("test1.rs"),
                warning_type: crate::error::WarningType::SyntaxError,
                message: "Error 1".to_string(),
            },
            ParseWarning {
                file_path: PathBuf::from("test2.py"),
                warning_type: crate::error::WarningType::EncodingError,
                message: "Error 2".to_string(),
            },
        ];
        let result = reporter.display_warnings(&warnings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_project_summary_with_methods() {
        use std::collections::HashMap;

        let reporter = TerminalReporter::new();

        // Create a summary with methods
        let summary = ProjectSummary {
            total_files: 2,
            total_lines: 300,
            total_functions: 10,
            total_methods: 5,
            total_classes: 3,
            language_breakdown: HashMap::new(),
            largest_files: vec![],
            most_complex_files: vec![],
        };

        let result = reporter.display_project_summary(&summary);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_cyclomatic_cell() {
        let reporter = TerminalReporter::new();

        // Test low complexity (green)
        let cell = reporter.format_cyclomatic_cell(5);
        assert!(cell.get_content().contains("5"));

        // Test medium complexity (yellow)
        let cell = reporter.format_cyclomatic_cell(15);
        assert!(cell.get_content().contains("15"));

        // Test high complexity (red)
        let cell = reporter.format_cyclomatic_cell(25);
        assert!(cell.get_content().contains("25"));
    }

    #[test]
    fn test_display_top_files() {
        let files = create_test_file_analysis();
        let summary = crate::analyzer::parser::create_project_summary(&files);

        let reporter = TerminalReporter::new();
        let result = reporter.display_top_files(&summary);
        assert!(result.is_ok());
    }
}
