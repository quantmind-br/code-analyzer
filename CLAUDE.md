# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Code Analyzer is a CLI tool written in Rust for analyzing codebases to identify refactoring candidates using AST parsing. It supports multiple languages (Rust, JavaScript, Python, Java, C, C++, Go, TypeScript) and provides both terminal and JSON output for CI/CD integration.

## Development Commands

### Building & Testing
```bash
# Development build
cargo build

# Release build (optimized with LTO)
cargo build --release

# Run all tests (unit + integration)
cargo test

# Run integration tests only
cargo test --test integration_tests

# Fast syntax check during development
cargo check
```

### Code Quality (Always run before committing)
```bash
# Full quality check - ALL must pass before committing
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Individual checks:
cargo fmt          # Format code
cargo fmt --check  # Check formatting
cargo clippy       # Linting
cargo clippy -- -D warnings  # Strict linting for CI
```

### Running the Tool
```bash
# Basic analysis
cargo run

# With options
cargo run -- --min-lines 100 --sort complexity --output json

# Using installed binary
cargo install --path .
code-analyzer --help
```

## Architecture

### Module Structure
- **src/lib.rs** - Main library API and analysis orchestration
- **src/analyzer/** - Core analysis engine using tree-sitter
  - `mod.rs` - AnalyzerEngine and AnalysisStats
  - `language.rs` - Language detection and file extensions
  - `parser.rs` - AST parsing with tree-sitter grammars
  - `walker.rs` - File system traversal with gitignore support
- **src/output/** - Dual output system (terminal + JSON)
  - `terminal.rs` - Pretty table formatting
  - `json.rs` - JSON serialization for CI/CD
- **src/cli.rs** - Command-line interface using clap derive API
- **src/error.rs** - Centralized error handling with AnalyzerError enum

### Key Dependencies
- **tree-sitter** + language grammars for AST parsing
- **rayon** for parallel file processing
- **clap** for CLI argument parsing
- **prettytable-rs** for terminal output
- **ignore** crate for gitignore support

## Development Guidelines

### Testing Strategy
- Integration tests in `tests/integration_tests.rs` test CLI functionality end-to-end
- Unit tests embedded in modules test individual components
- Use `tempfile` for creating test projects
- Test with sample files in `test_mixed_languages/` directory

### Error Handling
- Use `AnalyzerError` enum for all errors
- Implement `From` traits for automatic error conversion
- Return `Result<T>` for fallible operations

### Performance Considerations
- File analysis runs in parallel using rayon
- Use `--release` builds for performance testing
- Tree-sitter parsing is the main performance bottleneck

### Windows Compatibility
- Paths use Windows separators (`\`) on Windows
- CLI testing works with Windows command prompt
- Git operations standard across platforms

## Task Completion Checklist

Before committing any changes, ensure:
1. ✅ `cargo fmt --check` passes
2. ✅ `cargo clippy -- -D warnings` passes  
3. ✅ `cargo test` passes (all tests)
4. ✅ `cargo build --release` succeeds

## Language Support

Adding new language support requires:
1. Add tree-sitter grammar dependency to Cargo.toml
2. Update language detection in `src/analyzer/language.rs`
3. Add file extension mappings
4. Test with sample files