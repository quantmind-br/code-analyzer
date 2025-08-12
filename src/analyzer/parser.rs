use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Tree};

use super::language::{LanguageManager, NodeKindMapper, SupportedLanguage};
use crate::error::{AnalyzerError, Result};

/// Analysis result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub language: String,
    pub lines_of_code: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub functions: usize,
    pub classes: usize,
    pub complexity_score: f64,
}

impl FileAnalysis {
    /// Get total lines (code + blank + comments)
    pub fn total_lines(&self) -> usize {
        self.lines_of_code + self.blank_lines + self.comment_lines
    }

    /// Calculate a simple complexity score based on functions and classes
    pub fn calculate_complexity(&mut self) {
        let base_score = self.lines_of_code as f64 / 100.0;
        let function_multiplier = (self.functions as f64).sqrt() * 0.5;
        let class_multiplier = (self.classes as f64).sqrt() * 0.3;

        self.complexity_score = base_score + function_multiplier + class_multiplier;
    }
}

/// Complete analysis report structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisReport {
    pub files: Vec<FileAnalysis>,
    pub summary: ProjectSummary,
    pub config: AnalysisConfig,
    pub generated_at: DateTime<Utc>,
}

/// Project-wide summary statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectSummary {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_classes: usize,
    pub language_breakdown: HashMap<String, LanguageStats>,
    pub largest_files: Vec<FileAnalysis>,
    pub most_complex_files: Vec<FileAnalysis>,
}

/// Statistics for a specific language
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanguageStats {
    pub file_count: usize,
    pub total_lines: usize,
    pub avg_functions_per_file: f64,
    pub avg_classes_per_file: f64,
}

/// Configuration used for the analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisConfig {
    pub target_path: PathBuf,
    pub languages: Vec<String>,
    pub min_lines: usize,
    pub max_lines: Option<usize>,
    pub include_hidden: bool,
    pub max_file_size_mb: usize,
}

/// Core file parser using tree-sitter
#[derive(Clone)]
pub struct FileParser {
    language_manager: LanguageManager,
    max_file_size_bytes: u64,
}

impl FileParser {
    /// Create a new file parser
    pub fn new(language_manager: LanguageManager, max_file_size_mb: usize) -> Self {
        Self {
            language_manager,
            max_file_size_bytes: max_file_size_mb as u64 * 1024 * 1024,
        }
    }

    /// Parse a single file and extract metrics
    pub fn parse_file_metrics<P: AsRef<Path>>(&mut self, path: P) -> Result<FileAnalysis> {
        let path = path.as_ref();

        // Detect language
        let language = self.language_manager.detect_language(path).ok_or_else(|| {
            AnalyzerError::unsupported_language(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown"),
            )
        })?;

        // Check file size before reading
        let metadata = fs::metadata(path)?;
        if metadata.len() > self.max_file_size_bytes {
            return Err(AnalyzerError::validation_error(format!(
                "File too large: {} bytes",
                metadata.len()
            )));
        }

        // Read file contents
        let source_code = fs::read(path)?;

        // Validate UTF-8 encoding
        let source_text = std::str::from_utf8(&source_code)
            .map_err(|e| AnalyzerError::parse_error(format!("Invalid UTF-8 encoding: {e}")))?;

        // Parse with tree-sitter
        let parser = self.language_manager.get_parser(language)?;
        let tree = parse_file_safely(parser, &source_code)?;

        // Count lines
        let line_counts = count_lines(source_text);

        // Extract AST metrics
        let functions = if let Some(ref tree) = tree {
            count_functions(&tree.root_node(), &source_code, &language)
        } else {
            0
        };

        let classes = if let Some(ref tree) = tree {
            count_classes(&tree.root_node(), &source_code, &language)
        } else {
            0
        };

        let mut analysis = FileAnalysis {
            path: path.to_path_buf(),
            language: language.to_string(),
            lines_of_code: line_counts.code,
            blank_lines: line_counts.blank,
            comment_lines: line_counts.comments,
            functions,
            classes,
            complexity_score: 0.0,
        };

        // Calculate complexity score
        analysis.calculate_complexity();

        Ok(analysis)
    }

    /// Check if a file can be parsed (size and language support)
    pub fn can_parse<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();

        // Check if language is supported
        if !self.language_manager.is_supported_file(path) {
            return false;
        }

        // Check file size
        if let Ok(metadata) = fs::metadata(path) {
            metadata.len() <= self.max_file_size_bytes
        } else {
            false
        }
    }

    /// Get statistics about the language manager
    pub fn language_stats(&self) -> HashMap<SupportedLanguage, usize> {
        self.language_manager.parser_stats()
    }

    /// Get the maximum file size in bytes
    pub fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size_bytes
    }
}

/// Line counting results
#[derive(Debug)]
struct LineCounts {
    code: usize,
    blank: usize,
    comments: usize,
}

/// Count different types of lines in source code
fn count_lines(source: &str) -> LineCounts {
    let mut code_lines = 0;
    let mut blank_lines = 0;
    let mut comment_lines = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            blank_lines += 1;
        } else if is_comment_line(trimmed) {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    LineCounts {
        code: code_lines,
        blank: blank_lines,
        comments: comment_lines,
    }
}

/// Basic heuristic to detect comment lines
fn is_comment_line(line: &str) -> bool {
    line.starts_with("//")  // C-style, Rust, JavaScript, etc.
        || line.starts_with('#')   // Python, shell, etc.
        || line.starts_with("/*")  // C-style block comment start
        || line.starts_with("*/")  // C-style block comment end
        || line.starts_with("*")   // C-style block comment continuation
        || line.starts_with("<!--") // HTML comment
        || line.starts_with("-->") // HTML comment end
}

/// Safely parse a file with error recovery
fn parse_file_safely(parser: &mut tree_sitter::Parser, source: &[u8]) -> Result<Option<Tree>> {
    match parser.parse(source, None) {
        Some(tree) => {
            if tree.root_node().has_error() {
                // Log the error but continue with partial results
                eprintln!("Parse errors detected in file, results may be incomplete");
            }
            Ok(Some(tree))
        }
        None => Err(AnalyzerError::tree_sitter_error(
            "Failed to parse file - tree-sitter returned None",
        )),
    }
}

/// Count function declarations in an AST tree
fn count_functions(node: &Node, _source: &[u8], language: &SupportedLanguage) -> usize {
    let mut count = 0;

    // Check if this node is a function
    if language.is_function_node(node.kind()) {
        count += 1;
    }

    // Recursively traverse children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count += count_functions(&child, _source, language);
    }

    count
}

/// Count class/struct declarations in an AST tree
fn count_classes(node: &Node, _source: &[u8], language: &SupportedLanguage) -> usize {
    let mut count = 0;

    // Check if this node is a class/struct
    if language.is_class_node(node.kind()) {
        count += 1;
    }

    // Recursively traverse children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count += count_classes(&child, _source, language);
    }

    count
}

/// Create a project summary from analysis results
pub fn create_project_summary(files: &[FileAnalysis]) -> ProjectSummary {
    let total_files = files.len();
    let total_lines = files.iter().map(|f| f.total_lines()).sum();
    let total_functions = files.iter().map(|f| f.functions).sum();
    let total_classes = files.iter().map(|f| f.classes).sum();

    // Calculate language breakdown
    let mut language_breakdown = HashMap::new();
    for file in files {
        let entry = language_breakdown
            .entry(file.language.clone())
            .or_insert_with(|| LanguageStats {
                file_count: 0,
                total_lines: 0,
                avg_functions_per_file: 0.0,
                avg_classes_per_file: 0.0,
            });

        entry.file_count += 1;
        entry.total_lines += file.total_lines();
    }

    // Calculate averages for each language
    for (lang, stats) in language_breakdown.iter_mut() {
        let lang_files: Vec<_> = files.iter().filter(|f| f.language == *lang).collect();
        let total_functions: usize = lang_files.iter().map(|f| f.functions).sum();
        let total_classes: usize = lang_files.iter().map(|f| f.classes).sum();

        stats.avg_functions_per_file = if stats.file_count > 0 {
            total_functions as f64 / stats.file_count as f64
        } else {
            0.0
        };

        stats.avg_classes_per_file = if stats.file_count > 0 {
            total_classes as f64 / stats.file_count as f64
        } else {
            0.0
        };
    }

    // Get largest files (top 10 by total lines)
    let mut largest_files = files.to_vec();
    largest_files.sort_by_key(|f| std::cmp::Reverse(f.total_lines()));
    largest_files.truncate(10);

    // Get most complex files (top 10 by complexity score)
    let mut most_complex_files = files.to_vec();
    most_complex_files.sort_by(|a, b| b.complexity_score.partial_cmp(&a.complexity_score).unwrap());
    most_complex_files.truncate(10);

    ProjectSummary {
        total_files,
        total_lines,
        total_functions,
        total_classes,
        language_breakdown,
        largest_files,
        most_complex_files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_counting() {
        let source = r#"
// This is a comment
fn main() {
    println!("Hello, world!");
    
    // Another comment
}

/* Block comment */
        "#;

        let counts = count_lines(source);
        assert!(counts.code > 0);
        assert!(counts.blank > 0);
        assert!(counts.comments > 0);
    }

    #[test]
    fn test_is_comment_line() {
        assert!(is_comment_line("// Single line comment"));
        assert!(is_comment_line("# Python comment"));
        assert!(is_comment_line("/* Block comment */"));
        assert!(!is_comment_line("let x = 5;"));
        assert!(!is_comment_line("    code_with_spaces"));
    }

    #[test]
    fn test_file_analysis_complexity() {
        let mut analysis = FileAnalysis {
            path: PathBuf::from("test.rs"),
            language: "rust".to_string(),
            lines_of_code: 100,
            blank_lines: 10,
            comment_lines: 20,
            functions: 5,
            classes: 2,
            complexity_score: 0.0,
        };

        analysis.calculate_complexity();
        assert!(analysis.complexity_score > 0.0);
    }

    #[test]
    fn test_create_project_summary() {
        let files = vec![
            FileAnalysis {
                path: PathBuf::from("test1.rs"),
                language: "rust".to_string(),
                lines_of_code: 100,
                blank_lines: 10,
                comment_lines: 5,
                functions: 3,
                classes: 1,
                complexity_score: 2.5,
            },
            FileAnalysis {
                path: PathBuf::from("test2.rs"),
                language: "rust".to_string(),
                lines_of_code: 200,
                blank_lines: 20,
                comment_lines: 10,
                functions: 5,
                classes: 2,
                complexity_score: 4.0,
            },
        ];

        let summary = create_project_summary(&files);
        assert_eq!(summary.total_files, 2);
        assert_eq!(summary.total_lines, 345); // 100+10+5 + 200+20+10
        assert_eq!(summary.total_functions, 8);
        assert_eq!(summary.total_classes, 3);
        assert!(summary.language_breakdown.contains_key("rust"));

        let rust_stats = &summary.language_breakdown["rust"];
        assert_eq!(rust_stats.file_count, 2);
        assert_eq!(rust_stats.avg_functions_per_file, 4.0); // 8/2
    }
}
