---
name: "Code Analyzer CLI Tool - Complete Rust Implementation"
description: "A comprehensive CLI tool for analyzing codebases to identify refactoring candidates using AST parsing across multiple programming languages"
---

## Goal

**Feature Goal**: Create a production-ready CLI tool that analyzes codebases recursively, counts lines/functions/classes per file using AST parsing, respects .gitignore rules, and generates both terminal and JSON reports to help developers prioritize refactoring efforts.

**Deliverable**: Complete Rust CLI application `code-analyzer` with multi-language AST parsing, parallel file processing, terminal table output, JSON export, and comprehensive error handling.

**Success Definition**: 
- Analyzes codebases with 10K+ files in under 30 seconds
- Supports 8+ programming languages with accurate AST-based metrics
- Generates professional terminal tables and structured JSON reports
- Handles malformed code gracefully without crashing
- Follows Rust CLI best practices with comprehensive error messages

## User Persona

**Target User**: Senior Developer / Software Architect

**Use Case**: Assessing technical debt and planning refactoring priorities for large codebases

**User Journey**: 
1. Run `code-analyzer` in project directory to get quick overview
2. Use `code-analyzer --min-lines 100 --sort functions` to identify complex files
3. Export `refactor-candidates.json` for integration with CI/CD pipelines
4. Use filtering options to focus analysis on specific languages or patterns

**Pain Points Addressed**: 
- Manual code review is time-consuming and inconsistent
- Existing tools are language-specific or lack precision
- Need both human-readable reports and machine-readable data
- Large codebases require fast, reliable analysis

## Why

- **Business Value**: Reduces technical debt assessment time from days to minutes
- **Integration**: Structured JSON output enables CI/CD pipeline integration
- **Accuracy**: AST parsing provides precise metrics vs heuristic approaches
- **Developer Experience**: Professional CLI design with clear progress reporting and error handling

## What

A Rust CLI tool that recursively analyzes directory trees, parsing source files with tree-sitter AST parsers, counting lines/functions/classes with language-specific accuracy, filtering files using .gitignore rules, and outputting both formatted terminal tables and structured JSON reports.

### Success Criteria

- [ ] **Multi-Language Support**: Accurately parse and analyze 8+ languages (Rust, JavaScript/TypeScript, Python, Java, C/C++, Go)
- [ ] **Performance**: Process 10,000+ files in under 30 seconds with parallel processing
- [ ] **Output Quality**: Professional terminal tables with sorting/filtering + structured JSON export
- [ ] **Reliability**: Handle malformed code and file system errors without crashing
- [ ] **User Experience**: Intuitive CLI with progress reporting, help text, and meaningful error messages
- [ ] **Gitignore Compliance**: Respect .gitignore, .ignore, and custom exclude patterns

## All Needed Context

### Context Completeness Check

_This PRP provides complete implementation guidance for someone unfamiliar with Rust CLI development, tree-sitter AST parsing, and multi-language code analysis tools._

### Documentation & References

```yaml
- url: https://docs.rs/clap/latest/clap/
  why: CLI argument parsing with derive API, validation patterns, help generation
  critical: Derive API setup, argument validation, error handling patterns

- url: https://docs.rs/tree-sitter/latest/tree_sitter/
  why: AST parsing library for multi-language support, node traversal patterns
  critical: Parser initialization, language grammar loading, error recovery

- url: https://docs.rs/ignore/latest/ignore/
  why: Gitignore-aware directory traversal with parallel processing capabilities
  critical: WalkBuilder configuration, parallel iteration, custom filtering

- url: https://docs.rs/prettytable-rs/latest/prettytable/
  why: Terminal table formatting with alignment, styling, and performance
  critical: Table creation macros, formatting options, large dataset handling

- url: https://github.com/XAMPPRocky/tokei
  why: Reference implementation of multi-language code analysis tool
  critical: Project structure, language detection, performance optimization

- docfile: PRPs/ai_docs/tree_sitter_multi_language.md
  why: Comprehensive AST parsing patterns for function/class detection across languages
  section: "AST Traversal Patterns" and "Multi-Language Support Pattern"

- docfile: PRPs/ai_docs/rust_cli_patterns.md  
  why: Rust CLI development best practices, error handling, project structure
  section: "Project Structure Best Practices" and "Error Handling Patterns"
```

### Current Codebase Tree

```bash
E:\code-analyser\
├── PLAN.md              # Project specification (completed)
├── PRPs\
│   ├── README.md        # PRP system documentation
│   ├── ai_docs\
│   │   ├── tree_sitter_multi_language.md  # AST parsing guide
│   │   └── rust_cli_patterns.md          # CLI development patterns
│   └── templates\       # PRP templates
└── (empty - new Rust project)
```

### Desired Codebase Tree with Files to be Added

```bash
E:\code-analyser\
├── Cargo.toml           # Project dependencies and metadata
├── Cargo.lock           # Dependency lock file
├── README.md            # User documentation and installation guide
├── src\
│   ├── main.rs          # CLI entry point and orchestration
│   ├── lib.rs           # Library interface for testability
│   ├── cli.rs           # CLI argument definitions with clap
│   ├── error.rs         # Comprehensive error types and handling
│   ├── config.rs        # Configuration management (optional .toml support)
│   ├── analyzer\
│   │   ├── mod.rs       # Analyzer module interface
│   │   ├── parser.rs    # File parsing with tree-sitter integration
│   │   ├── language.rs  # Language detection and AST node mapping
│   │   ├── metrics.rs   # Line/function/class counting logic
│   │   └── walker.rs    # Directory traversal with ignore crate
│   └── output\
│       ├── mod.rs       # Output module interface
│       ├── terminal.rs  # Terminal table formatting with prettytable-rs
│       ├── json.rs      # JSON export with serde_json
│       └── report.rs    # Unified reporting data structures
├── tests\
│   ├── integration_tests.rs  # End-to-end CLI testing
│   └── fixtures\        # Test code samples in various languages
└── examples\
    └── sample_analysis.json  # Example output format
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Tree-sitter language grammars must be compatible versions
// Each language parser is a separate crate with specific version requirements
// Example: tree-sitter v0.20 requires matching language parser versions

// CRITICAL: ignore crate WalkBuilder must be configured before iteration
// Parallel iteration requires Arc<Mutex<>> for thread-safe result collection
let mut builder = WalkBuilder::new(path);
builder.hidden(false).parallel(); // Configure before .build()

// CRITICAL: prettytable-rs macros require specific import pattern
use prettytable::{Table, Row, Cell, row, cell, table};

// CRITICAL: Clap derive API requires feature flag in Cargo.toml
// [dependencies]
// clap = { version = "4.0", features = ["derive"] }

// GOTCHA: Tree-sitter nodes have different types per language
// Must handle language-specific node kinds for functions/classes
// JavaScript: "function_declaration" vs Python: "function_definition"

// GOTCHA: Large files can cause memory pressure with tree-sitter
// Implement file size limits or streaming for files >10MB

// GOTCHA: Binary files will cause parse errors
// Filter by file extension before attempting AST parsing
```

## Implementation Blueprint

### Data Models and Structure

Core data models ensuring type safety and consistency across parsing, analysis, and output.

```rust
// Core analysis result structure
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub files: Vec<FileAnalysis>,
    pub summary: ProjectSummary,
    pub config: AnalysisConfig,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_functions: usize,
    pub total_classes: usize,
    pub language_breakdown: HashMap<String, LanguageStats>,
    pub largest_files: Vec<FileAnalysis>,  // Top 10 by lines
    pub most_complex_files: Vec<FileAnalysis>, // Top 10 by functions
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageStats {
    pub file_count: usize,
    pub total_lines: usize,
    pub avg_functions_per_file: f64,
    pub avg_classes_per_file: f64,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE Cargo.toml and project initialization
  - IMPLEMENT: Rust CLI project with dependencies for clap, tree-sitter, ignore, prettytable-rs, serde_json
  - DEPENDENCIES: clap = { version = "4.0", features = ["derive"] }, tree-sitter = "0.20", multiple tree-sitter language crates
  - NAMING: code-analyzer package name, version = "0.1.0"
  - PLACEMENT: Root directory

Task 2: CREATE src/error.rs
  - IMPLEMENT: AnalyzerError enum with variants for Io, Parse, InvalidPath, UnsupportedLanguage, TreeSitter
  - FOLLOW pattern: Rust error handling best practices with Display, Error traits, From implementations
  - NAMING: AnalyzerError with descriptive variant names, Result<T> type alias
  - PLACEMENT: Core error handling for entire application

Task 3: CREATE src/cli.rs  
  - IMPLEMENT: CliArgs struct with clap derive API for argument parsing
  - FOLLOW pattern: Comprehensive CLI design with positional args, optional flags, validation
  - NAMING: CliArgs with descriptive field names, SortBy and OutputFormat enums
  - DEPENDENCIES: clap derive macros, path validation, enum value parsing
  - PLACEMENT: CLI interface definition

Task 4: CREATE src/analyzer/language.rs
  - IMPLEMENT: Language detection by file extension and tree-sitter parser management  
  - FOLLOW pattern: Multi-language support with parser caching and error recovery
  - NAMING: SupportedLanguage enum, get_language_parser function, NodeKindMapper trait
  - DEPENDENCIES: tree-sitter language crates, file extension mapping
  - PLACEMENT: Language-specific parsing logic

Task 5: CREATE src/analyzer/parser.rs
  - IMPLEMENT: File parsing with tree-sitter, AST traversal for metrics collection
  - FOLLOW pattern: PRPs/ai_docs/tree_sitter_multi_language.md AST traversal patterns
  - NAMING: FileParser struct, parse_file_metrics method, count_functions/count_classes functions
  - DEPENDENCIES: Task 4 language detection, tree-sitter Parser, error handling
  - PLACEMENT: Core AST parsing and metrics extraction

Task 6: CREATE src/analyzer/walker.rs  
  - IMPLEMENT: Directory traversal with gitignore support using ignore crate
  - FOLLOW pattern: Parallel file discovery with custom filtering and progress reporting
  - NAMING: FileWalker struct, discover_files method, FilterConfig struct
  - DEPENDENCIES: ignore crate WalkBuilder, custom file filtering, parallel processing
  - PLACEMENT: File discovery and traversal logic

Task 7: CREATE src/output/terminal.rs
  - IMPLEMENT: Terminal table formatting with prettytable-rs, sorting, and styling
  - FOLLOW pattern: Professional table design with alignment, headers, summary rows
  - NAMING: TerminalReporter struct, format_analysis_table method, apply_sorting function
  - DEPENDENCIES: prettytable-rs table macros, FileAnalysis data structures
  - PLACEMENT: Terminal output formatting

Task 8: CREATE src/output/json.rs
  - IMPLEMENT: JSON export functionality with serde_json, pretty printing options
  - FOLLOW pattern: Structured JSON output with comprehensive metadata
  - NAMING: JsonExporter struct, export_to_file method, format_json function  
  - DEPENDENCIES: serde_json serialization, AnalysisReport structure, file I/O
  - PLACEMENT: JSON export functionality

Task 9: CREATE src/analyzer/mod.rs
  - IMPLEMENT: Analyzer module interface coordinating parsing, walking, and metrics
  - FOLLOW pattern: Orchestration layer managing parallel file processing
  - NAMING: AnalyzerEngine struct, analyze_project method, progress reporting
  - DEPENDENCIES: Tasks 4-6 analyzer components, parallel processing coordination
  - PLACEMENT: High-level analysis orchestration

Task 10: CREATE src/output/mod.rs  
  - IMPLEMENT: Output module interface managing both terminal and JSON output
  - FOLLOW pattern: Unified output management with format switching
  - NAMING: OutputManager struct, generate_output method, dual format support
  - DEPENDENCIES: Tasks 7-8 output components, CLI arguments for format selection
  - PLACEMENT: Output coordination and format management

Task 11: CREATE src/lib.rs
  - IMPLEMENT: Library interface exposing core functionality for testing
  - FOLLOW pattern: Clean API design with error propagation and configuration
  - NAMING: run_analysis function, public re-exports, library interface
  - DEPENDENCIES: All previous tasks, comprehensive error handling
  - PLACEMENT: Library interface for main.rs and testing

Task 12: CREATE src/main.rs
  - IMPLEMENT: CLI entry point with argument parsing, logging, and error handling
  - FOLLOW pattern: PRPs/ai_docs/rust_cli_patterns.md main.rs minimal orchestration
  - NAMING: main function, init_logging, error formatting for CLI output
  - DEPENDENCIES: Task 11 library interface, Task 3 CLI args, comprehensive error display
  - PLACEMENT: Application entry point

Task 13: CREATE tests/integration_tests.rs
  - IMPLEMENT: End-to-end testing with temporary file fixtures and CLI invocation
  - FOLLOW pattern: Integration testing with real file samples and output validation
  - NAMING: test_analyze_rust_project, test_json_output_format, test_filtering_options
  - COVERAGE: CLI argument parsing, file analysis accuracy, output format validation
  - PLACEMENT: Integration testing suite
```

### Implementation Patterns & Key Details

```rust
// CRITICAL: Multi-language AST parsing pattern
use tree_sitter::{Language, Parser, Node};

// Language parser management with caching
pub struct LanguageManager {
    parsers: HashMap<SupportedLanguage, Parser>,
}

impl LanguageManager {
    pub fn get_parser(&mut self, lang: SupportedLanguage) -> Result<&mut Parser> {
        if !self.parsers.contains_key(&lang) {
            let mut parser = Parser::new();
            parser.set_language(&lang.get_grammar())?;
            self.parsers.insert(lang, parser);
        }
        Ok(self.parsers.get_mut(&lang).unwrap())
    }
}

// PATTERN: Parallel file processing with progress reporting
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn analyze_files_parallel(files: Vec<PathBuf>) -> Result<Vec<FileAnalysis>> {
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .progress_chars("#>-"));
    
    let results: Result<Vec<_>> = files.par_iter().map(|file| {
        let result = analyze_single_file(file);
        pb.inc(1);
        result
    }).collect();
    
    pb.finish_with_message("Analysis complete");
    results
}

// CRITICAL: Error recovery pattern for malformed code
fn parse_file_safely(parser: &mut Parser, source: &[u8]) -> Option<tree_sitter::Tree> {
    match parser.parse(source, None) {
        Some(tree) if !tree.root_node().has_error() => Some(tree),
        Some(_) => {
            eprintln!("Parse errors detected, attempting recovery");
            // Continue with partial results
            None
        }
        None => None,
    }
}

// PATTERN: Terminal table formatting with sorting
use prettytable::{Table, Row, Cell, row};

pub fn create_analysis_table(files: &[FileAnalysis], sort_by: SortBy) -> Table {
    let mut table = Table::new();
    table.add_row(row!["File", "Language", "Lines", "Functions", "Classes", "Complexity"]);
    
    let mut sorted_files = files.to_vec();
    match sort_by {
        SortBy::Lines => sorted_files.sort_by_key(|f| std::cmp::Reverse(f.lines_of_code)),
        SortBy::Functions => sorted_files.sort_by_key(|f| std::cmp::Reverse(f.functions)),
        SortBy::Classes => sorted_files.sort_by_key(|f| std::cmp::Reverse(f.classes)),
    }
    
    for file in sorted_files.iter().take(50) { // Limit display
        table.add_row(row![
            file.path.display(),
            file.language,
            file.lines_of_code,
            file.functions,
            file.classes,
            format!("{:.2}", file.complexity_score)
        ]);
    }
    table
}
```

### Integration Points

```yaml
CARGO_DEPENDENCIES:
  - clap: { version = "4.0", features = ["derive"] }
  - tree-sitter: "0.20"
  - tree-sitter-rust: "0.20"
  - tree-sitter-javascript: "0.20"
  - tree-sitter-python: "0.20"  
  - tree-sitter-java: "0.20"
  - tree-sitter-c: "0.20"
  - tree-sitter-cpp: "0.20"
  - tree-sitter-go: "0.20"
  - ignore: "0.4"
  - prettytable-rs: "0.10"
  - serde_json: "1.0"
  - serde: { version = "1.0", features = ["derive"] }
  - chrono: { version = "0.4", features = ["serde"] }
  - rayon: "1.8"  # Parallel processing
  - indicatif: "0.17"  # Progress bars

BINARY_CONFIG:
  - name: "code-analyzer"
  - path: "src/main.rs"

BUILD_FEATURES:
  - default: ["full-analysis"]
  - minimal: excludes some language parsers for faster builds
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Install Rust toolchain and format tools
rustup component add rustfmt clippy

# Run after each file creation - fix before proceeding
cargo fmt                           # Format code consistently
cargo clippy -- -D warnings        # Lint with warnings as errors
cargo check                         # Fast compilation check

# Expected: Zero errors and warnings. Clean, idiomatic Rust code.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test core functionality as components are created
cargo test analyzer::language::tests -v     # Language detection tests
cargo test analyzer::parser::tests -v       # AST parsing tests  
cargo test output::terminal::tests -v       # Table formatting tests
cargo test output::json::tests -v           # JSON export tests

# Full test suite
cargo test -v                               # All unit and integration tests

# Coverage validation (if llvm-cov installed)
cargo llvm-cov --html                       # Generate coverage report

# Expected: >90% test coverage, all tests pass, no panics in error cases
```

### Level 3: Integration Testing (System Validation)

```bash
# Build and install locally for testing
cargo build --release                       # Optimized build
cargo install --path .                     # Install to local cargo bin

# Basic functionality validation
code-analyzer --help                        # Help text displays correctly
code-analyzer tests/fixtures                # Analyze test code samples
code-analyzer tests/fixtures --json-only    # JSON-only output test
code-analyzer tests/fixtures --min-lines 50 --sort functions  # Filtering test

# Performance testing with larger codebase
time code-analyzer /path/to/large/project   # Performance benchmark
code-analyzer /path/to/large/project --verbose  # Detailed progress output

# Output validation
test -f refactor-candidates.json            # JSON file created
jq . refactor-candidates.json > /dev/null   # Valid JSON format

# Error handling validation
code-analyzer /nonexistent/path             # Graceful error handling
code-analyzer tests/fixtures/malformed_code # Parse error recovery

# Expected: <30s for 10K files, graceful error handling, valid JSON output
```

### Level 4: Real-World Validation

```bash
# Test with actual open-source projects
git clone https://github.com/rust-lang/cargo.git temp_test
code-analyzer temp_test --verbose          # Large Rust project analysis

git clone https://github.com/microsoft/vscode.git temp_test_js  
code-analyzer temp_test_js --languages javascript,typescript  # JS/TS project

# Comparison with existing tools (if available)
tokei temp_test                             # Compare line counts
cloc temp_test                              # Compare with cloc

# Memory and performance profiling
/usr/bin/time -v code-analyzer temp_test    # Memory usage analysis
perf record code-analyzer temp_test         # Performance profiling (Linux)

# Multi-language project validation
code-analyzer . --output both --sort functions  # Self-analysis of project

# Expected: Results match or exceed existing tools, efficient memory usage
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test -v`
- [ ] No linting errors: `cargo clippy -- -D warnings`
- [ ] No formatting issues: `cargo fmt -- --check`
- [ ] Performance target met: <30 seconds for 10K+ files

### Feature Validation

- [ ] Multi-language support: 8+ languages with accurate AST parsing
- [ ] Terminal output: Professional tables with sorting and filtering
- [ ] JSON export: Valid, comprehensive structured data
- [ ] Gitignore compliance: Respects .gitignore and custom patterns
- [ ] Error handling: Graceful degradation with informative messages
- [ ] Progress reporting: Clear feedback during long operations

### Code Quality Validation

- [ ] Follows Rust best practices and idioms
- [ ] Comprehensive error handling without panics
- [ ] Memory efficient for large codebases
- [ ] Clean separation of concerns across modules
- [ ] Extensive test coverage with edge cases
- [ ] Performance optimized with parallel processing

### User Experience Validation

- [ ] Intuitive CLI interface matching conventional patterns
- [ ] Clear help text and error messages
- [ ] Reasonable defaults with powerful customization options
- [ ] Professional output formatting in both terminal and JSON
- [ ] Installation works via `cargo install --path .`

---

## Anti-Patterns to Avoid

- ❌ Don't use regex/heuristics for code analysis - use proper AST parsing
- ❌ Don't panic on malformed code - recover gracefully and continue
- ❌ Don't load all files into memory - process streaming for large codebases
- ❌ Don't ignore performance - use parallel processing and efficient data structures
- ❌ Don't hardcode language lists - make easily extensible
- ❌ Don't skip error context - provide actionable error messages
- ❌ Don't output directly from parsing - use structured data models
- ❌ Don't block on single file failures - continue processing other files