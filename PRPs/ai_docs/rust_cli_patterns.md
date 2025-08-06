# Rust CLI Development Patterns Guide

## Project Structure Best Practices

### Standard Layout for CLI Tools
```
src/
├── main.rs          # Entry point, argument parsing, orchestration
├── lib.rs           # Core library functions (if reusable)
├── cli.rs           # CLI argument definitions and validation
├── analyzer/        # Core analysis logic
│   ├── mod.rs
│   ├── parser.rs    # File parsing logic
│   ├── counter.rs   # Metric counting
│   └── language.rs  # Language detection
├── output/          # Output formatting
│   ├── mod.rs
│   ├── terminal.rs  # Terminal table output
│   └── json.rs      # JSON export
└── error.rs         # Error types and handling
```

### main.rs vs lib.rs Pattern
```rust
// main.rs - Keep minimal, handle CLI and orchestration
use clap::Parser;
use code_analyzer::{CliArgs, run_analysis};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    run_analysis(args)?;
    Ok(())
}

// lib.rs - Core library logic for testability
pub mod cli;
pub mod analyzer;
pub mod output;
pub mod error;

pub use cli::CliArgs;
pub use error::AnalyzerError;

pub fn run_analysis(args: CliArgs) -> Result<(), AnalyzerError> {
    // Core analysis logic here
}
```

## Error Handling Patterns

### Comprehensive Error Types
```rust
use std::fmt;

#[derive(Debug)]
pub enum AnalyzerError {
    Io(std::io::Error),
    Parse(String),
    InvalidPath(std::path::PathBuf),
    UnsupportedLanguage(String),
    ConfigError(String),
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnalyzerError::Io(err) => write!(f, "IO error: {}", err),
            AnalyzerError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AnalyzerError::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            AnalyzerError::UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
            AnalyzerError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AnalyzerError {}

// Implement From for automatic conversions
impl From<std::io::Error> for AnalyzerError {
    fn from(err: std::io::Error) -> Self {
        AnalyzerError::Io(err)
    }
}
```

### Result Type Alias
```rust
pub type Result<T> = std::result::Result<T, AnalyzerError>;
```

## CLI Argument Patterns with Clap

### Comprehensive CLI Structure
```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "code-analyzer")]
#[command(about = "Analyze code for refactoring candidates")]
#[command(version)]
pub struct CliArgs {
    /// Directory to analyze (default: current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
    
    /// Minimum lines of code to include in results
    #[arg(long, default_value_t = 1)]
    pub min_lines: usize,
    
    /// Maximum lines of code to include in results  
    #[arg(long)]
    pub max_lines: Option<usize>,
    
    /// Sort results by [lines|functions|classes]
    #[arg(long, value_enum, default_value_t = SortBy::Lines)]
    pub sort: SortBy,
    
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
    
    /// Only output JSON file, no terminal output
    #[arg(long)]
    pub json_only: bool,
    
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Languages to analyze (default: all supported)
    #[arg(long, value_delimiter = ',')]
    pub languages: Vec<String>,
}

#[derive(clap::ValueEnum, Clone)]
pub enum SortBy {
    Lines,
    Functions, 
    Classes,
    Name,
}

#[derive(clap::ValueEnum, Clone)]
pub enum OutputFormat {
    Table,
    Json,
    Both,
}
```

## Performance Patterns

### Parallel File Processing
```rust
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

pub fn analyze_files_parallel(files: Vec<PathBuf>) -> Result<Vec<FileAnalysis>> {
    let results = Arc::new(Mutex::new(Vec::new()));
    
    files.par_iter().try_for_each(|file| -> Result<()> {
        match analyze_single_file(file) {
            Ok(analysis) => {
                results.lock().unwrap().push(analysis);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to analyze {}: {}", file.display(), e);
                Ok(()) // Continue processing other files
            }
        }
    })?;
    
    let mut results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
    results.sort_by_key(|a| a.lines);
    Ok(results)
}
```

### Progress Reporting
```rust
use indicatif::{ProgressBar, ProgressStyle};

pub fn analyze_with_progress(files: &[PathBuf]) -> Result<Vec<FileAnalysis>> {
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));
    
    let mut results = Vec::new();
    for file in files {
        results.push(analyze_single_file(file)?);
        pb.inc(1);
    }
    pb.finish_with_message("Analysis complete");
    Ok(results)
}
```

## Output Formatting Patterns

### Unified Data Structure
```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub language: String,
    pub lines: usize,
    pub functions: usize,
    pub classes: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub files: Vec<FileAnalysis>,
    pub summary: Summary,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_classes: usize,
    pub languages: std::collections::HashMap<String, usize>,
}
```

## Configuration Management

### Optional Config File Support
```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub languages: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub min_lines: usize,
    pub max_functions_per_file: usize,
}

impl Config {
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            languages: vec![
                "rust".to_string(),
                "javascript".to_string(), 
                "python".to_string(),
                "java".to_string(),
            ],
            exclude_patterns: vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                ".git/".to_string(),
            ],
            min_lines: 10,
            max_functions_per_file: 50,
        }
    }
}
```

## Testing Patterns

### Integration Testing Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.rs"), "fn test() {}\nstruct Test {}")
            .unwrap();
        dir
    }
    
    #[test]
    fn test_analyze_rust_file() {
        let dir = create_test_project();
        let result = analyze_single_file(&dir.path().join("test.rs")).unwrap();
        
        assert_eq!(result.functions, 1);
        assert_eq!(result.classes, 1);
        assert_eq!(result.language, "rust");
    }
}
```

## Logging and Debugging

### Structured Logging Setup
```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber;

pub fn init_logging(verbose: bool) {
    let level = if verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
}

// Usage in main
fn main() -> Result<()> {
    let args = CliArgs::parse();
    init_logging(args.verbose);
    
    info!("Starting analysis of {:?}", args.path);
    // ... rest of logic
}
```