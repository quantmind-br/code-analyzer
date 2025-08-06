# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Code Analyzer is a Rust CLI tool that analyzes codebases to identify refactoring candidates using AST parsing with tree-sitter. It supports multiple languages (Rust, JavaScript, Python, Java, C, C++, Go, TypeScript) and provides both terminal table output and JSON export for CI/CD integration.

## Essential Commands

### Development Workflow
```bash
# Build and check
cargo check                    # Quick syntax/type checking
cargo build                    # Debug build
cargo build --release         # Optimized release build

# Code quality (run in this order after changes)
cargo fmt                      # Format code
cargo clippy                   # Lint with clippy
cargo clippy -- -D warnings   # Lint with no warnings allowed (CI standard)

# Testing
cargo test                                  # Run all tests
cargo test --test integration_tests        # Run integration tests only
cargo test -- --nocapture                  # Show output from tests
cargo test test_name                        # Run specific test

# Local installation for testing
cargo install --path .
```

### Usage Examples
```bash
# Basic analysis
cargo run
cargo run -- --help
cargo run -- path/to/analyze --verbose

# With filters and options
cargo run -- --min-lines 100 --languages rust,python --sort complexity
cargo run -- --output json --limit 20
```

## Architecture Overview

The codebase follows a modular architecture with clear separation of concerns:

### Core Components

**`src/lib.rs`** - Main orchestration layer that coordinates the entire analysis workflow:
1. Validates CLI arguments
2. Creates and configures AnalyzerEngine
3. Runs analysis on target directory
4. Generates output in requested format(s)

**`src/analyzer/`** - Analysis engine with three key modules:
- `language.rs` - Language detection and management (SupportedLanguage enum, LanguageManager)
- `parser.rs` - AST parsing using tree-sitter parsers (FileParser, AnalysisReport, ProjectSummary)
- `walker.rs` - File system traversal with gitignore support (FileWalker, FilterConfig)

**`src/output/`** - Dual output system:
- `terminal.rs` - Pretty table formatting using prettytable-rs
- `json.rs` - Structured JSON export for CI/CD integration
- `mod.rs` - OutputManager that coordinates both outputs

**`src/cli.rs`** - CLI interface using clap derive API with comprehensive filtering options

**`src/error.rs`** - Custom error types and error handling utilities

### Key Design Patterns

**AnalyzerEngine** - Central orchestrator that composes language manager, file parser, and file walker components. Created either with defaults or from CLI arguments.

**Language Management** - Extensible language support through SupportedLanguage enum and tree-sitter grammar integration. Defaults to stable languages (Rust, Python, JavaScript, TypeScript) rather than all supported.

**Parallel Processing** - Uses `rayon` for CPU-intensive file processing with `indicatif` progress reporting.

**Dual Output Strategy** - Generates both human-readable terminal tables and machine-readable JSON simultaneously for different use cases.

## Testing Strategy

**Integration Tests** (`tests/integration_tests.rs`):
- Creates temporary test projects with multi-language source files
- Tests complete analysis workflow from CLI args to output
- Uses `tempfile` for isolated test environments
- Tests both API functions and CLI command execution

**Test File Structure**:
- Creates realistic test projects with Rust, Python, JavaScript files
- Includes complex scenarios (nested functions, classes, modules)
- Tests filtering, sorting, and output format options

## Project Requirements Prompts (PRPs)

The `PRPs/` directory contains Product Requirement Prompts - structured prompts for AI-driven development that combine traditional PRD scope with AI-specific context, implementation details, and validation gates. This approach ensures focused, deliverable software increments with proper context for code generation.

## Development Notes

- **Tree-sitter Integration**: Language parsers are loaded dynamically; add new languages by extending SupportedLanguage enum and adding corresponding tree-sitter grammar
- **Performance**: Uses parallel processing for file analysis; progress reporting available with `--verbose` flag
- **Error Handling**: Comprehensive error types in `error.rs`; analysis continues on individual file failures
- **Output Flexibility**: Support for terminal tables, JSON export, or both simultaneously
- **File Filtering**: Respects .gitignore rules plus additional exclude patterns and file size limits