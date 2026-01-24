use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::cli::CliArgs;
use crate::error::{AnalyzerError, ParseWarning, Result};

pub mod git;
pub mod language;
pub mod parser;
pub mod walker;

pub use git::{get_changed_files, get_repo_root, is_git_repository};
pub use language::{LanguageManager, SupportedLanguage};
pub use parser::{
    create_project_summary, identify_refactoring_candidates, AnalysisConfig, AnalysisReport,
    FileAnalysis, FileAnalysisResult, FileParser, ProjectSummary, RefactoringCandidate,
    RefactoringReason, RefactoringThresholds,
};
pub use walker::{create_walker_from_cli, FileWalker, FilterConfig, WalkStats};

/// Core analyzer engine that orchestrates the analysis process
pub struct AnalyzerEngine {
    language_manager: LanguageManager,
    file_parser: FileParser,
    file_walker: FileWalker,
    show_progress: bool,
}

impl AnalyzerEngine {
    /// Create a new analyzer engine with default configuration
    pub fn new() -> Self {
        let language_manager = LanguageManager::new();
        let file_parser = FileParser::new(LanguageManager::new(), 10); // 10MB default
        let file_walker = FileWalker::new(LanguageManager::new());

        Self {
            language_manager,
            file_parser,
            file_walker,
            show_progress: false,
        }
    }

    /// Create an analyzer engine from CLI arguments
    pub fn from_cli_args(args: &CliArgs) -> Result<Self> {
        // Validate and parse target languages
        let target_languages = if args.languages.is_empty() {
            // Use all supported languages by default for comprehensive analysis
            SupportedLanguage::all()
        } else {
            language::validate_language_list(&args.languages)?
        };

        // Create base language manager (created once, cloned for components that need their own copy)
        let base_language_manager = LanguageManager::with_languages(target_languages);

        // Create file parser with size limits (needs own LanguageManager for thread-safety)
        let file_parser = FileParser::new(base_language_manager.clone(), args.max_file_size_mb);

        // Create file walker from CLI args (needs own LanguageManager for language detection)
        let file_walker = create_walker_from_cli(args, base_language_manager.clone());

        Ok(Self {
            language_manager: base_language_manager,
            file_parser,
            file_walker,
            show_progress: args.verbose,
        })
    }

    /// Analyze a project directory and return comprehensive results
    pub fn analyze_project<P: AsRef<Path>>(
        &mut self,
        target_path: P,
        cli_args: &CliArgs,
    ) -> Result<AnalysisReport> {
        let target_path = target_path.as_ref();

        if self.show_progress {
            println!("Starting analysis of: {}", target_path.display());
        }

        // Step 1: Discover files (with optional git filtering)
        let (files, walk_stats) = if let Some(ref commit_ref) = cli_args.only_changed_since {
            self.discover_git_changed_files(target_path, commit_ref)?
        } else {
            self.file_walker.discover_files(target_path)?
        };

        if self.show_progress {
            println!("Discovered {} files to analyze", files.len());
            println!("Walk statistics: {}", walk_stats.summary());
        }

        if files.is_empty() {
            return Err(AnalyzerError::validation_error(
                "No supported files found in the specified directory",
            ));
        }

        // Step 2: Analyze files in parallel (now returns warnings too)
        let (analysis_results, warnings) = self.analyze_files_parallel(&files)?;

        // Step 3: Apply CLI filters
        let filtered_results = self.apply_cli_filters(analysis_results, cli_args);

        // Step 4: Create project summary
        let summary = create_project_summary(&filtered_results);

        // Step 5: Create analysis configuration record
        let config = AnalysisConfig {
            target_path: target_path.to_path_buf(),
            languages: cli_args.languages.clone(),
            min_lines: cli_args.min_lines,
            max_lines: cli_args.max_lines,
            include_hidden: cli_args.include_hidden,
            max_file_size_mb: cli_args.max_file_size_mb,
        };

        // Step 6: Create final report with warnings
        let report = AnalysisReport {
            files: filtered_results,
            summary,
            config,
            generated_at: Utc::now(),
            warnings,
        };

        if self.show_progress {
            println!("Analysis completed successfully");
            println!("Total files analyzed: {}", report.files.len());
            println!("Total lines of code: {}", report.summary.total_lines);
        }

        Ok(report)
    }

    /// Result from parallel file analysis
    fn analyze_files_parallel(
        &mut self,
        files: &[std::path::PathBuf],
    ) -> Result<(Vec<FileAnalysis>, Vec<ParseWarning>)> {
        let progress_bar = if self.show_progress {
            let pb = ProgressBar::new(files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("#>-"),
            );
            pb.set_message("Analyzing files...");
            Some(pb)
        } else {
            None
        };

        // Use thread-safe containers for results, warnings, and errors
        let results = Arc::new(Mutex::new(Vec::new()));
        let warnings = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        let show_progress = self.show_progress;
        let max_file_size_bytes = self.file_parser.max_file_size_bytes();
        let enabled_languages = self.language_manager.enabled_languages();

        // Parallel analysis with progress reporting
        files.par_iter().for_each(|file| {
            // Update progress
            if let Some(ref pb) = progress_bar {
                pb.set_message(format!(
                    "Analyzing: {}",
                    file.file_name().unwrap_or_default().to_string_lossy()
                ));
                pb.inc(1);
            }

            // Create a thread-local parser for this file
            let language_manager = LanguageManager::with_languages(enabled_languages.clone());
            let mut file_parser = FileParser::new(
                language_manager,
                (max_file_size_bytes / (1024 * 1024)) as usize,
            );

            // Analyze single file with warnings
            match file_parser.parse_file_with_warnings(file) {
                Ok(result) => {
                    results
                        .lock()
                        .expect("analysis results mutex poisoned")
                        .push(result.analysis);
                    if !result.warnings.is_empty() {
                        if show_progress {
                            for warning in &result.warnings {
                                eprintln!("Warning: {}", warning);
                                for loc in warning.locations.iter().take(3) {
                                    if let Some(snippet) = &loc.snippet {
                                        eprintln!(
                                            "  at {}:{} ({})  {}",
                                            loc.line, loc.column, loc.kind, snippet
                                        );
                                    } else {
                                        eprintln!(
                                            "  at {}:{} ({})",
                                            loc.line, loc.column, loc.kind
                                        );
                                    }
                                }
                            }
                        }
                        warnings
                            .lock()
                            .expect("parse warnings mutex poisoned")
                            .extend(result.warnings);
                    }
                }
                Err(e) => {
                    if show_progress {
                        eprintln!("Warning: Failed to analyze {}: {}", file.display(), e);
                    }
                    errors
                        .lock()
                        .expect("analysis errors mutex poisoned")
                        .push((file.clone(), e));
                }
            }
        });

        if let Some(pb) = progress_bar {
            pb.finish_with_message("File analysis completed");
        }

        // Extract results
        let results = Arc::try_unwrap(results)
            .expect("results Arc still has multiple owners")
            .into_inner()
            .expect("results mutex poisoned");
        let warnings = Arc::try_unwrap(warnings)
            .expect("warnings Arc still has multiple owners")
            .into_inner()
            .expect("warnings mutex poisoned");
        let errors = Arc::try_unwrap(errors)
            .expect("errors Arc still has multiple owners")
            .into_inner()
            .expect("errors mutex poisoned");

        // Report errors if in verbose mode
        if self.show_progress && !errors.is_empty() {
            println!("Encountered {} errors during analysis:", errors.len());
            for (file, error) in &errors {
                eprintln!("  {}: {}", file.display(), error);
            }
        }

        if results.is_empty() {
            return Err(AnalyzerError::validation_error(
                "Failed to analyze any files successfully",
            ));
        }

        Ok((results, warnings))
    }

    /// Apply CLI-based filters to analysis results
    fn apply_cli_filters(
        &self,
        mut results: Vec<FileAnalysis>,
        cli_args: &CliArgs,
    ) -> Vec<FileAnalysis> {
        // Filter by minimum lines
        results.retain(|analysis| analysis.lines_of_code >= cli_args.min_lines);

        // Filter by maximum lines if specified
        if let Some(max_lines) = cli_args.max_lines {
            results.retain(|analysis| analysis.lines_of_code <= max_lines);
        }

        // Filter by minimum functions if specified
        if let Some(min_functions) = cli_args.min_functions {
            results.retain(|analysis| analysis.functions >= min_functions);
        }

        // Filter by minimum classes if specified
        if let Some(min_classes) = cli_args.min_classes {
            results.retain(|analysis| analysis.classes >= min_classes);
        }

        results
    }

    /// Get analysis statistics
    pub fn get_analysis_stats(&self) -> AnalysisStats {
        AnalysisStats {
            supported_languages: self.language_manager.enabled_languages(),
            parser_stats: self.file_parser.language_stats(),
            filter_config: self.file_walker.filter_config().clone(),
        }
    }

    /// Enable or disable progress reporting
    pub fn set_show_progress(&mut self, show: bool) {
        self.show_progress = show;
    }

    /// Update the maximum file size for analysis
    pub fn set_max_file_size_mb(&mut self, size_mb: usize) {
        self.file_parser = FileParser::new(
            LanguageManager::with_languages(self.language_manager.enabled_languages()),
            size_mb,
        );
    }

    /// Discover only files changed since a git commit
    fn discover_git_changed_files<P: AsRef<Path>>(
        &self,
        target_path: P,
        commit_ref: &str,
    ) -> Result<(Vec<std::path::PathBuf>, WalkStats)> {
        let target_path = target_path.as_ref();

        if self.show_progress {
            println!("Git mode: analyzing files changed since '{}'", commit_ref);
        }

        // Get changed files from git
        let changed_files = git::get_changed_files(target_path, commit_ref)?;

        if self.show_progress {
            println!("Git reports {} changed files", changed_files.len());
        }

        // Filter through the normal language/size filters
        let filtered_files: Vec<std::path::PathBuf> = changed_files
            .into_iter()
            .filter(|f| self.file_parser.can_parse(f))
            .collect();

        let stats = WalkStats {
            files_found: filtered_files.len(),
            total_entries_scanned: filtered_files.len(),
            directories_scanned: 0,
            files_skipped_size: 0,
            files_skipped_language: 0,
            files_skipped_hidden: 0,
            errors_encountered: 0,
        };

        if self.show_progress {
            println!(
                "After filtering: {} supported files to analyze",
                filtered_files.len()
            );
        }

        Ok((filtered_files, stats))
    }
}

impl Default for AnalyzerEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the analyzer engine configuration
#[derive(Debug)]
pub struct AnalysisStats {
    pub supported_languages: Vec<SupportedLanguage>,
    pub parser_stats: std::collections::HashMap<SupportedLanguage, usize>,
    pub filter_config: FilterConfig,
}

impl AnalysisStats {
    /// Get a summary of the analyzer configuration
    pub fn summary(&self) -> String {
        format!(
            "Languages: {}, Parsers initialized: {}, Max file size: {}MB",
            self.supported_languages.len(),
            self.parser_stats.len(),
            self.filter_config.max_file_size_bytes / (1024 * 1024)
        )
    }
}

/// Convenience function to analyze a project with default settings
pub fn analyze_project_simple<P: AsRef<Path>>(
    target_path: P,
    languages: Option<Vec<String>>,
    max_file_size_mb: Option<usize>,
) -> Result<AnalysisReport> {
    let cli_args = CliArgs {
        path: Some(target_path.as_ref().to_path_buf()),
        languages: languages.unwrap_or_default(), // Empty vec triggers all languages in from_cli_args
        max_file_size_mb: max_file_size_mb.unwrap_or(10),
        verbose: false,
        ..Default::default()
    };

    let mut engine = AnalyzerEngine::from_cli_args(&cli_args)?;
    engine.analyze_project(target_path, &cli_args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create test files
        fs::write(
            root.join("main.rs"),
            r#"
            fn main() {
                println!("Hello, world!");
            }
            
            struct TestStruct {
                value: i32,
            }
            
            impl TestStruct {
                fn new(value: i32) -> Self {
                    Self { value }
                }
                
                fn get_value(&self) -> i32 {
                    self.value
                }
            }
            "#,
        )
        .unwrap();

        fs::write(
            root.join("utils.py"),
            r#"
            def hello_world():
                print("Hello from Python!")
                
            class Calculator:
                def __init__(self):
                    self.result = 0
                    
                def add(self, value):
                    self.result += value
                    return self
            "#,
        )
        .unwrap();

        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn test() {}").unwrap();

        dir
    }

    #[test]
    fn test_analyzer_engine_creation() {
        let engine = AnalyzerEngine::new();
        let stats = engine.get_analysis_stats();

        assert!(!stats.supported_languages.is_empty());
        assert!(stats.summary().contains("Languages:"));
    }

    #[test]
    fn test_analyzer_engine_from_cli() {
        let cli_args = CliArgs {
            languages: vec!["rust".to_string()],
            max_file_size_mb: 5,
            verbose: true,
            ..Default::default()
        };

        let engine = AnalyzerEngine::from_cli_args(&cli_args);
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        let stats = engine.get_analysis_stats();
        assert!(stats.supported_languages.contains(&SupportedLanguage::Rust));
    }

    #[test]
    fn test_analyze_project() {
        let test_dir = create_test_project();
        let cli_args = CliArgs {
            path: Some(test_dir.path().to_path_buf()),
            verbose: false,
            ..Default::default()
        };

        let mut engine = AnalyzerEngine::from_cli_args(&cli_args).unwrap();
        let result = engine.analyze_project(test_dir.path(), &cli_args);

        if result.is_err() {
            eprintln!(
                "test_analyze_project failed with error: {:?}",
                result.as_ref().unwrap_err()
            );
        }
        assert!(result.is_ok());
        let report = result.unwrap();

        // Should find at least the Rust and Python files
        assert!(report.files.len() >= 2);
        assert!(report.summary.total_functions > 0);
        assert!(report.summary.total_lines > 0);

        // Check language breakdown
        assert!(report.summary.language_breakdown.contains_key("rust"));
        assert!(report.summary.language_breakdown.contains_key("python"));
    }

    #[test]
    fn test_analyze_project_simple() {
        let test_dir = create_test_project();

        let result =
            analyze_project_simple(test_dir.path(), Some(vec!["rust".to_string()]), Some(5));

        assert!(result.is_ok());
        let report = result.unwrap();

        // Should only find Rust files
        assert!(report.files.iter().all(|f| f.language == "rust"));
    }

    #[test]
    fn test_cli_filters() {
        let test_dir = create_test_project();
        let cli_args = CliArgs {
            path: Some(test_dir.path().to_path_buf()),
            min_lines: 5,         // Filter out very small files
            max_lines: Some(100), // Filter out very large files
            verbose: false,
            ..Default::default()
        };

        let mut engine = AnalyzerEngine::from_cli_args(&cli_args).unwrap();
        let result = engine.analyze_project(test_dir.path(), &cli_args);

        if result.is_err() {
            eprintln!(
                "test_cli_filters failed with error: {:?}",
                result.as_ref().unwrap_err()
            );
        }
        assert!(result.is_ok());
        let report = result.unwrap();

        // All files should meet the line requirements
        for file in &report.files {
            assert!(file.lines_of_code >= 5);
            assert!(file.lines_of_code <= 100);
        }
    }

    #[test]
    fn test_get_analysis_stats() {
        let engine = AnalyzerEngine::new();
        let stats = engine.get_analysis_stats();

        assert!(!stats.supported_languages.is_empty());
        let summary = stats.summary();
        assert!(summary.contains("Languages:"));
        assert!(summary.contains("MB"));
    }
}
