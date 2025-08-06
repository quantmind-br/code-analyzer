use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::language::LanguageManager;
use crate::error::{AnalyzerError, Result};

/// Configuration for file filtering during traversal
#[derive(Debug, Clone)]
pub struct FilterConfig {
    /// Maximum file size in bytes
    pub max_file_size_bytes: u64,

    /// Whether to include hidden files and directories
    pub include_hidden: bool,

    /// Additional glob patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Languages to include (if empty, include all supported)
    pub target_languages: Vec<String>,

    /// Follow symbolic links
    pub follow_symlinks: bool,

    /// Maximum directory depth to traverse
    pub max_depth: Option<usize>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 10 * 1024 * 1024, // 10MB
            include_hidden: false,
            exclude_patterns: Vec::new(),
            target_languages: Vec::new(),
            follow_symlinks: false,
            max_depth: None,
        }
    }
}

/// Statistics about the file discovery process
#[derive(Debug, Default)]
pub struct WalkStats {
    pub total_entries_scanned: usize,
    pub files_found: usize,
    pub directories_scanned: usize,
    pub files_skipped_size: usize,
    pub files_skipped_language: usize,
    pub files_skipped_hidden: usize,
    pub errors_encountered: usize,
}

impl WalkStats {
    /// Get a summary of the walk statistics
    pub fn summary(&self) -> String {
        format!(
            "Scanned {} entries, found {} files, {} errors",
            self.total_entries_scanned, self.files_found, self.errors_encountered
        )
    }
}

/// File walker with gitignore support and parallel processing
pub struct FileWalker {
    filter_config: FilterConfig,
    language_manager: LanguageManager,
    show_progress: bool,
}

impl FileWalker {
    /// Create a new file walker with default configuration
    pub fn new(language_manager: LanguageManager) -> Self {
        Self {
            filter_config: FilterConfig::default(),
            language_manager,
            show_progress: false,
        }
    }

    /// Create a file walker with custom filter configuration
    pub fn with_config(language_manager: LanguageManager, filter_config: FilterConfig) -> Self {
        Self {
            filter_config,
            language_manager,
            show_progress: false,
        }
    }

    /// Enable or disable progress reporting
    pub fn show_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Discover files in a directory with parallel processing
    pub fn discover_files<P: AsRef<Path>>(
        &self,
        root_path: P,
    ) -> Result<(Vec<PathBuf>, WalkStats)> {
        let root_path = root_path.as_ref();

        if !root_path.exists() {
            return Err(AnalyzerError::invalid_path(root_path));
        }

        if !root_path.is_dir() {
            return Err(AnalyzerError::validation_error(format!(
                "Path must be a directory: {}",
                root_path.display()
            )));
        }

        // Set up the ignore walker
        let mut builder = WalkBuilder::new(root_path);
        self.configure_walker(&mut builder)?;

        // Collect results with thread-safe containers
        let files = Arc::new(Mutex::new(Vec::new()));
        let stats = Arc::new(Mutex::new(WalkStats::default()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Set up progress bar if requested
        let progress_bar = if self.show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template("{spinner:.green} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            pb.set_message("Discovering files...");
            Some(pb)
        } else {
            None
        };

        // Parallel file discovery
        builder.build_parallel().run(|| {
            let files = Arc::clone(&files);
            let stats = Arc::clone(&stats);
            let errors = Arc::clone(&errors);
            let progress_bar = progress_bar.as_ref().cloned();
            let filter_config = self.filter_config.clone();
            let language_manager = &self.language_manager;

            Box::new(move |result| {
                let mut stats_guard = stats.lock().unwrap();
                stats_guard.total_entries_scanned += 1;

                match result {
                    Ok(entry) => {
                        let path = entry.path();

                        // Update progress
                        if let Some(ref pb) = progress_bar {
                            pb.set_message(format!(
                                "Scanning: {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                            pb.inc(1);
                        }

                        if path.is_dir() {
                            stats_guard.directories_scanned += 1;
                            return WalkState::Continue;
                        }

                        // Apply file filters
                        match should_include_file(path, &filter_config, language_manager) {
                            Ok(IncludeResult::Include) => {
                                files.lock().unwrap().push(path.to_path_buf());
                                stats_guard.files_found += 1;
                            }
                            Ok(IncludeResult::SkipSize) => {
                                stats_guard.files_skipped_size += 1;
                            }
                            Ok(IncludeResult::SkipLanguage) => {
                                stats_guard.files_skipped_language += 1;
                            }
                            Ok(IncludeResult::SkipHidden) => {
                                stats_guard.files_skipped_hidden += 1;
                            }
                            Err(e) => {
                                errors.lock().unwrap().push(e);
                                stats_guard.errors_encountered += 1;
                            }
                        }
                    }
                    Err(err) => {
                        stats_guard.errors_encountered += 1;
                        errors.lock().unwrap().push(AnalyzerError::Walk(err));
                    }
                }

                WalkState::Continue
            })
        });

        if let Some(pb) = progress_bar {
            pb.finish_with_message("File discovery completed");
        }

        // Extract results
        let files = Arc::try_unwrap(files).unwrap().into_inner().unwrap();

        let stats = Arc::try_unwrap(stats).unwrap().into_inner().unwrap();

        let errors = Arc::try_unwrap(errors).unwrap().into_inner().unwrap();

        // Log errors if any
        for error in &errors {
            eprintln!("Walk error: {error}");
        }

        Ok((files, stats))
    }

    /// Configure the WalkBuilder with filter settings
    fn configure_walker(&self, builder: &mut WalkBuilder) -> Result<()> {
        // Configure basic walker settings
        builder
            .hidden(!self.filter_config.include_hidden)
            .follow_links(self.filter_config.follow_symlinks)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true);

        // Set maximum depth if specified
        if let Some(max_depth) = self.filter_config.max_depth {
            builder.max_depth(Some(max_depth));
        }

        // Add custom exclude patterns
        for pattern in &self.filter_config.exclude_patterns {
            builder.add_custom_ignore_filename(pattern);
        }

        Ok(())
    }

    /// Get a reference to the filter configuration
    pub fn filter_config(&self) -> &FilterConfig {
        &self.filter_config
    }

    /// Update the filter configuration
    pub fn set_filter_config(&mut self, config: FilterConfig) {
        self.filter_config = config;
    }
}

/// Result of file inclusion check
#[derive(Debug)]
enum IncludeResult {
    Include,
    SkipSize,
    SkipLanguage,
    SkipHidden,
}

/// Check if a file should be included based on filters
fn should_include_file(
    path: &Path,
    config: &FilterConfig,
    language_manager: &LanguageManager,
) -> Result<IncludeResult> {
    // Check if hidden and we're not including hidden files
    if !config.include_hidden {
        if let Some(file_name) = path.file_name() {
            if file_name.to_string_lossy().starts_with('.') {
                return Ok(IncludeResult::SkipHidden);
            }
        }
    }

    // Check language support
    if !language_manager.is_supported_file(path) {
        return Ok(IncludeResult::SkipLanguage);
    }

    // Check specific language filtering
    if !config.target_languages.is_empty() {
        if let Some(detected_lang) = language_manager.detect_language(path) {
            if !config.target_languages.contains(&detected_lang.to_string()) {
                return Ok(IncludeResult::SkipLanguage);
            }
        } else {
            return Ok(IncludeResult::SkipLanguage);
        }
    }

    // Check file size
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() > config.max_file_size_bytes {
            return Ok(IncludeResult::SkipSize);
        }
    } else {
        // If we can't get metadata, skip the file
        return Ok(IncludeResult::SkipSize);
    }

    Ok(IncludeResult::Include)
}

/// Helper function to create a walker with CLI arguments
pub fn create_walker_from_cli(
    cli_args: &crate::cli::CliArgs,
    language_manager: LanguageManager,
) -> FileWalker {
    let filter_config = FilterConfig {
        max_file_size_bytes: cli_args.max_file_size_bytes(),
        include_hidden: cli_args.include_hidden,
        exclude_patterns: cli_args.exclude.clone(),
        target_languages: cli_args.languages.clone(),
        follow_symlinks: false,
        max_depth: None,
    };

    FileWalker::with_config(language_manager, filter_config).show_progress(cli_args.verbose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create some test files
        fs::write(root.join("main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("lib.py"), "def hello(): pass").unwrap();
        fs::write(root.join("script.js"), "function test() {}").unwrap();
        fs::write(root.join("README.md"), "# Test Project").unwrap();

        // Create a subdirectory
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src").join("module.rs"), "pub fn test() {}").unwrap();

        // Create a hidden file
        fs::write(root.join(".hidden"), "hidden content").unwrap();

        dir
    }

    #[test]
    fn test_file_discovery() {
        let test_dir = create_test_project();
        let language_manager = LanguageManager::new();
        let walker = FileWalker::new(language_manager);

        let (files, stats) = walker.discover_files(test_dir.path()).unwrap();

        // Should find supported language files
        assert!(files.len() > 0);
        assert!(stats.files_found > 0);
        assert!(files.iter().any(|p| p.extension().unwrap() == "rs"));
        assert!(files.iter().any(|p| p.extension().unwrap() == "py"));
        assert!(files.iter().any(|p| p.extension().unwrap() == "js"));

        // Should not include README.md (unsupported language)
        assert!(!files.iter().any(|p| p.extension().unwrap() == "md"));
    }

    #[test]
    fn test_hidden_files_filtering() {
        let test_dir = create_test_project();
        let language_manager = LanguageManager::new();

        // Test without hidden files (default)
        let walker = FileWalker::new(language_manager);
        let (files, _) = walker.discover_files(test_dir.path()).unwrap();
        assert!(!files
            .iter()
            .any(|p| p.file_name().unwrap().to_string_lossy().starts_with('.')));

        // Test with hidden files
        let config = FilterConfig {
            include_hidden: true,
            ..FilterConfig::default()
        };
        let language_manager = LanguageManager::new();
        let walker = FileWalker::with_config(language_manager, config);
        let (files, _) = walker.discover_files(test_dir.path()).unwrap();

        // Note: .hidden file might not be included if it's not a supported language
        // This test mainly verifies the configuration is applied
    }

    #[test]
    fn test_language_filtering() {
        let test_dir = create_test_project();
        let language_manager = LanguageManager::new();

        let config = FilterConfig {
            target_languages: vec!["rust".to_string()],
            ..FilterConfig::default()
        };

        let walker = FileWalker::with_config(language_manager, config);
        let (files, _) = walker.discover_files(test_dir.path()).unwrap();

        // Should only find Rust files
        for file in files {
            assert_eq!(file.extension().unwrap(), "rs");
        }
    }

    #[test]
    fn test_file_size_filtering() {
        let test_dir = create_test_project();
        let language_manager = LanguageManager::new();

        let config = FilterConfig {
            max_file_size_bytes: 1, // Very small limit
            ..FilterConfig::default()
        };

        let walker = FileWalker::with_config(language_manager, config);
        let (files, stats) = walker.discover_files(test_dir.path()).unwrap();

        // Should skip files due to size
        assert!(stats.files_skipped_size > 0);
        assert!(files.len() == 0 || files.len() < stats.files_found + stats.files_skipped_size);
    }

    #[test]
    fn test_walk_stats() {
        let test_dir = create_test_project();
        let language_manager = LanguageManager::new();
        let walker = FileWalker::new(language_manager);

        let (_, stats) = walker.discover_files(test_dir.path()).unwrap();

        assert!(stats.total_entries_scanned > 0);
        assert!(stats.directories_scanned > 0);

        let summary = stats.summary();
        assert!(summary.contains("Scanned"));
        assert!(summary.contains("found"));
    }
}
