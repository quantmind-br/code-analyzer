use serde_json;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::analyzer::parser::{AnalysisReport, FileAnalysis, ProjectSummary};
use crate::cli::SortBy;
use crate::error::{AnalyzerError, Result};

/// JSON exporter for analysis results
#[derive(Clone)]
pub struct JsonExporter {
    pretty_print: bool,
    include_metadata: bool,
}

impl JsonExporter {
    /// Create a new JSON exporter with default settings
    pub fn new() -> Self {
        Self {
            pretty_print: true,
            include_metadata: true,
        }
    }

    /// Enable or disable pretty printing (formatted JSON)
    pub fn pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    /// Enable or disable metadata inclusion
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Export analysis report to a JSON file
    pub fn export_to_file<P: AsRef<Path>>(
        &self,
        report: &AnalysisReport,
        file_path: P,
    ) -> Result<()> {
        let json_content = self.format_json(report)?;

        // Write to file
        let mut file = fs::File::create(&file_path)?;
        file.write_all(json_content.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Format analysis report as JSON string
    pub fn format_json(&self, report: &AnalysisReport) -> Result<String> {
        let json = if self.pretty_print {
            serde_json::to_string_pretty(report)?
        } else {
            serde_json::to_string(report)?
        };

        Ok(json)
    }

    /// Export only file analysis data (without full report structure)
    pub fn export_files_only<P: AsRef<Path>>(
        &self,
        files: &[FileAnalysis],
        file_path: P,
    ) -> Result<()> {
        let json = if self.pretty_print {
            serde_json::to_string_pretty(files)?
        } else {
            serde_json::to_string(files)?
        };

        let mut file = fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Export project summary only
    pub fn export_summary_only<P: AsRef<Path>>(
        &self,
        summary: &ProjectSummary,
        file_path: P,
    ) -> Result<()> {
        let json = if self.pretty_print {
            serde_json::to_string_pretty(summary)?
        } else {
            serde_json::to_string(summary)?
        };

        let mut file = fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Create a filtered report with only files matching certain criteria
    pub fn export_filtered_report<P: AsRef<Path>>(
        &self,
        report: &AnalysisReport,
        file_path: P,
        sort_by: SortBy,
        limit: Option<usize>,
        min_lines: Option<usize>,
        min_functions: Option<usize>,
    ) -> Result<()> {
        let mut filtered_files = report.files.clone();

        // Apply filters
        if let Some(min_lines) = min_lines {
            filtered_files.retain(|f| f.lines_of_code >= min_lines);
        }

        if let Some(min_functions) = min_functions {
            filtered_files.retain(|f| f.functions >= min_functions);
        }

        // Apply sorting
        crate::output::terminal::apply_sorting(&mut filtered_files, sort_by);

        // Apply limit if specified
        if let Some(limit) = limit {
            filtered_files.truncate(limit);
        }

        // Create filtered report
        let filtered_report = AnalysisReport {
            files: filtered_files,
            summary: report.summary.clone(),
            config: report.config.clone(),
            generated_at: report.generated_at,
        };

        self.export_to_file(&filtered_report, file_path)
    }

    /// Validate JSON content by attempting to parse it
    pub fn validate_json_file<P: AsRef<Path>>(file_path: P) -> Result<()> {
        let content = fs::read_to_string(&file_path)?;

        // Try to parse as AnalysisReport
        serde_json::from_str::<AnalysisReport>(&content).map_err(AnalyzerError::Json)?;

        Ok(())
    }

    /// Read and parse an analysis report from JSON file
    pub fn import_from_file<P: AsRef<Path>>(file_path: P) -> Result<AnalysisReport> {
        let content = fs::read_to_string(&file_path)?;
        let report: AnalysisReport = serde_json::from_str(&content)?;
        Ok(report)
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick export function for common use cases
pub fn export_analysis_results<P: AsRef<Path>>(
    files: &[FileAnalysis],
    summary: &ProjectSummary,
    config: &crate::analyzer::parser::AnalysisConfig,
    file_path: P,
    pretty_print: bool,
) -> Result<()> {
    let report = AnalysisReport {
        files: files.to_vec(),
        summary: summary.clone(),
        config: config.clone(),
        generated_at: chrono::Utc::now(),
    };

    let exporter = JsonExporter::new().pretty_print(pretty_print);
    exporter.export_to_file(&report, file_path)
}

/// Export files in a compact format (without full report structure)
pub fn export_compact_json<P: AsRef<Path>>(files: &[FileAnalysis], file_path: P) -> Result<()> {
    let exporter = JsonExporter::new()
        .pretty_print(false)
        .include_metadata(false);

    exporter.export_files_only(files, file_path)
}

/// Create JSON string from analysis results without writing to file
pub fn format_analysis_json(
    files: &[FileAnalysis],
    summary: &ProjectSummary,
    config: &crate::analyzer::parser::AnalysisConfig,
    pretty: bool,
) -> Result<String> {
    let report = AnalysisReport {
        files: files.to_vec(),
        summary: summary.clone(),
        config: config.clone(),
        generated_at: chrono::Utc::now(),
    };

    let exporter = JsonExporter::new().pretty_print(pretty);
    exporter.format_json(&report)
}

/// Utility function to convert file analysis to JSON for API responses
pub fn files_to_json_value(files: &[FileAnalysis]) -> Result<serde_json::Value> {
    serde_json::to_value(files).map_err(AnalyzerError::Json)
}

/// Utility function to merge multiple analysis reports
pub fn merge_analysis_reports(reports: &[AnalysisReport]) -> Result<AnalysisReport> {
    if reports.is_empty() {
        return Err(AnalyzerError::validation_error("No reports to merge"));
    }

    let mut merged_files = Vec::new();

    // Merge all files
    for report in reports {
        merged_files.extend_from_slice(&report.files);
    }

    // Create merged summary
    let merged_summary = crate::analyzer::parser::create_project_summary(&merged_files);

    // Use the configuration from the first report
    let base_report = &reports[0];

    Ok(AnalysisReport {
        files: merged_files,
        summary: merged_summary,
        config: base_report.config.clone(),
        generated_at: chrono::Utc::now(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_test_analysis_report() -> AnalysisReport {
        let files = vec![
            FileAnalysis {
                path: PathBuf::from("src/main.rs"),
                language: "rust".to_string(),
                lines_of_code: 100,
                blank_lines: 10,
                comment_lines: 20,
                functions: 5,
                classes: 2,
                complexity_score: 3.2,
            },
            FileAnalysis {
                path: PathBuf::from("lib/utils.js"),
                language: "javascript".to_string(),
                lines_of_code: 75,
                blank_lines: 8,
                comment_lines: 15,
                functions: 3,
                classes: 1,
                complexity_score: 2.1,
            },
        ];

        let summary = crate::analyzer::parser::create_project_summary(&files);

        let config = crate::analyzer::parser::AnalysisConfig {
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
        }
    }

    #[test]
    fn test_json_export_to_file() {
        let report = create_test_analysis_report();
        let temp_file = NamedTempFile::new().unwrap();

        let exporter = JsonExporter::new();
        let result = exporter.export_to_file(&report, temp_file.path());

        assert!(result.is_ok());

        // Verify file was created and contains valid JSON
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("files"));
        assert!(content.contains("summary"));
    }

    #[test]
    fn test_format_json() {
        let report = create_test_analysis_report();
        let exporter = JsonExporter::new().pretty_print(true);

        let json = exporter.format_json(&report).unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("{\n")); // Pretty printed should have newlines

        let exporter_compact = JsonExporter::new().pretty_print(false);
        let json_compact = exporter_compact.format_json(&report).unwrap();
        assert!(json_compact.len() < json.len()); // Compact should be shorter
    }

    #[test]
    fn test_export_files_only() {
        let report = create_test_analysis_report();
        let temp_file = NamedTempFile::new().unwrap();

        let exporter = JsonExporter::new();
        let result = exporter.export_files_only(&report.files, temp_file.path());

        assert!(result.is_ok());

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.starts_with('[')); // Should be an array
        assert!(content.contains("main.rs"));
    }

    #[test]
    fn test_export_filtered_report() {
        let report = create_test_analysis_report();
        let temp_file = NamedTempFile::new().unwrap();

        let exporter = JsonExporter::new();
        let result = exporter.export_filtered_report(
            &report,
            temp_file.path(),
            SortBy::Lines,
            Some(1),  // Limit to 1 file
            Some(80), // Min lines = 80
            None,
        );

        assert!(result.is_ok());

        // Read and verify the filtered report
        let filtered_report = JsonExporter::import_from_file(temp_file.path()).unwrap();
        assert_eq!(filtered_report.files.len(), 1);
        assert!(filtered_report.files[0].lines_of_code >= 80);
    }

    #[test]
    fn test_validate_json_file() {
        let report = create_test_analysis_report();
        let temp_file = NamedTempFile::new().unwrap();

        let exporter = JsonExporter::new();
        exporter.export_to_file(&report, temp_file.path()).unwrap();

        // Valid file should pass validation
        let result = JsonExporter::validate_json_file(temp_file.path());
        assert!(result.is_ok());

        // Invalid JSON should fail
        fs::write(temp_file.path(), "invalid json").unwrap();
        let result = JsonExporter::validate_json_file(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_import_from_file() {
        let original_report = create_test_analysis_report();
        let temp_file = NamedTempFile::new().unwrap();

        // Export then import
        let exporter = JsonExporter::new();
        exporter
            .export_to_file(&original_report, temp_file.path())
            .unwrap();

        let imported_report = JsonExporter::import_from_file(temp_file.path()).unwrap();

        // Verify data integrity
        assert_eq!(imported_report.files.len(), original_report.files.len());
        assert_eq!(imported_report.files[0].path, original_report.files[0].path);
        assert_eq!(
            imported_report.files[0].language,
            original_report.files[0].language
        );
    }

    #[test]
    fn test_quick_export_functions() {
        let report = create_test_analysis_report();
        let temp_file = NamedTempFile::new().unwrap();

        // Test export_analysis_results
        let result = export_analysis_results(
            &report.files,
            &report.summary,
            &report.config,
            temp_file.path(),
            true,
        );
        assert!(result.is_ok());

        // Test export_compact_json
        let temp_file2 = NamedTempFile::new().unwrap();
        let result = export_compact_json(&report.files, temp_file2.path());
        assert!(result.is_ok());

        // Compact should be smaller
        let normal_size = fs::metadata(temp_file.path()).unwrap().len();
        let compact_size = fs::metadata(temp_file2.path()).unwrap().len();
        assert!(compact_size <= normal_size);
    }

    #[test]
    fn test_format_analysis_json() {
        let report = create_test_analysis_report();

        let json =
            format_analysis_json(&report.files, &report.summary, &report.config, true).unwrap();

        assert!(!json.is_empty());
        assert!(json.contains("files"));
        assert!(json.contains("summary"));
    }

    #[test]
    fn test_files_to_json_value() {
        let report = create_test_analysis_report();

        let json_value = files_to_json_value(&report.files).unwrap();
        assert!(json_value.is_array());

        let array = json_value.as_array().unwrap();
        assert_eq!(array.len(), 2);
    }

    #[test]
    fn test_merge_analysis_reports() {
        let report1 = create_test_analysis_report();
        let report2 = create_test_analysis_report();

        let merged = merge_analysis_reports(&[report1.clone(), report2]).unwrap();

        // Should have files from both reports
        assert_eq!(merged.files.len(), report1.files.len() * 2);
        assert!(merged.summary.total_files >= report1.summary.total_files);
    }
}
