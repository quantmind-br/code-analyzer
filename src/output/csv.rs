use csv::Writer;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use crate::analyzer::FileAnalysis;
use crate::error::Result;

#[derive(Clone)]
pub struct CsvExporter;

impl CsvExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export_to_file<P: AsRef<Path>>(&self, files: &[FileAnalysis], path: P) -> Result<()> {
        let file = File::create(path.as_ref())?;
        self.write_csv(files, file)
    }

    pub fn export_to_stdout(&self, files: &[FileAnalysis]) -> Result<()> {
        self.write_csv(files, io::stdout())
    }

    fn write_csv<W: Write>(&self, files: &[FileAnalysis], writer: W) -> Result<()> {
        let mut wtr = Writer::from_writer(writer);

        wtr.write_record([
            "path",
            "language",
            "lines_of_code",
            "blank_lines",
            "comment_lines",
            "functions",
            "methods",
            "classes",
            "cyclomatic_complexity",
            "max_nesting_depth",
            "complexity_score",
        ])?;

        for f in files {
            wtr.write_record([
                f.path.display().to_string(),
                f.language.clone(),
                f.lines_of_code.to_string(),
                f.blank_lines.to_string(),
                f.comment_lines.to_string(),
                f.functions.to_string(),
                f.methods.to_string(),
                f.classes.to_string(),
                f.cyclomatic_complexity.to_string(),
                f.max_nesting_depth.to_string(),
                format!("{:.2}", f.complexity_score),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn format_csv(&self, files: &[FileAnalysis]) -> Result<String> {
        let mut wtr = Writer::from_writer(Vec::new());

        wtr.write_record([
            "path",
            "language",
            "lines_of_code",
            "blank_lines",
            "comment_lines",
            "functions",
            "methods",
            "classes",
            "cyclomatic_complexity",
            "max_nesting_depth",
            "complexity_score",
        ])?;

        for f in files {
            wtr.write_record([
                f.path.display().to_string(),
                f.language.clone(),
                f.lines_of_code.to_string(),
                f.blank_lines.to_string(),
                f.comment_lines.to_string(),
                f.functions.to_string(),
                f.methods.to_string(),
                f.classes.to_string(),
                f.cyclomatic_complexity.to_string(),
                f.max_nesting_depth.to_string(),
                format!("{:.2}", f.complexity_score),
            ])?;
        }

        let data = wtr.into_inner().map_err(|e| {
            crate::error::AnalyzerError::validation_error(format!("CSV flush error: {}", e))
        })?;

        String::from_utf8(data).map_err(|e| {
            crate::error::AnalyzerError::validation_error(format!("CSV encoding error: {}", e))
        })
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn create_test_files() -> Vec<FileAnalysis> {
        vec![
            FileAnalysis {
                path: PathBuf::from("src/main.rs"),
                language: "rust".to_string(),
                lines_of_code: 100,
                blank_lines: 10,
                comment_lines: 20,
                functions: 5,
                methods: 3,
                classes: 2,
                cyclomatic_complexity: 8,
                max_nesting_depth: 3,
                complexity_score: 3.2,
            },
            FileAnalysis {
                path: PathBuf::from("lib/utils.js"),
                language: "javascript".to_string(),
                lines_of_code: 75,
                blank_lines: 8,
                comment_lines: 15,
                functions: 3,
                methods: 2,
                classes: 1,
                cyclomatic_complexity: 5,
                max_nesting_depth: 2,
                complexity_score: 2.1,
            },
        ]
    }

    #[test]
    fn test_csv_export_to_file() {
        let files = create_test_files();
        let temp_file = NamedTempFile::new().unwrap();
        let exporter = CsvExporter::new();

        let result = exporter.export_to_file(&files, temp_file.path());
        assert!(result.is_ok());

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("path,language,lines_of_code"));
        assert!(content.contains("src/main.rs,rust,100"));
        assert!(content.contains("lib/utils.js,javascript,75"));
    }

    #[test]
    fn test_csv_format_string() {
        let files = create_test_files();
        let exporter = CsvExporter::new();

        let result = exporter.format_csv(&files);
        assert!(result.is_ok());

        let csv_string = result.unwrap();
        assert!(csv_string.contains("path,language,lines_of_code"));
        assert!(csv_string.contains("max_nesting_depth,complexity_score"));
        assert!(csv_string.contains("src/main.rs"));
    }

    #[test]
    fn test_csv_empty_files() {
        let files: Vec<FileAnalysis> = vec![];
        let exporter = CsvExporter::new();

        let result = exporter.format_csv(&files);
        assert!(result.is_ok());

        let csv_string = result.unwrap();
        assert!(csv_string.contains("path,language,lines_of_code"));
        assert!(!csv_string.contains("src/main.rs"));
    }

    #[test]
    fn test_csv_all_fields_present() {
        let files = create_test_files();
        let exporter = CsvExporter::new();

        let csv_string = exporter.format_csv(&files).unwrap();
        let lines: Vec<&str> = csv_string.lines().collect();

        assert!(lines.len() >= 2);

        let header = lines[0];
        assert!(header.contains("path"));
        assert!(header.contains("language"));
        assert!(header.contains("lines_of_code"));
        assert!(header.contains("blank_lines"));
        assert!(header.contains("comment_lines"));
        assert!(header.contains("functions"));
        assert!(header.contains("methods"));
        assert!(header.contains("classes"));
        assert!(header.contains("cyclomatic_complexity"));
        assert!(header.contains("max_nesting_depth"));
        assert!(header.contains("complexity_score"));
    }
}
