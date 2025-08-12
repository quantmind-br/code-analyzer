# Code Structure & Architecture - Code Analyzer

## Project Layout
```
src/
├── main.rs                 # Entry point - minimal, delegates to lib
├── lib.rs                  # Main library with public API and analysis orchestration
├── cli.rs                  # CLI argument parsing and configuration
├── error.rs                # Centralized error handling with AnalyzerError enum
├── analyzer/               # Core analysis engine
│   ├── mod.rs             # AnalyzerEngine and AnalysisStats structs
│   ├── language.rs        # Language detection and extensions mapping
│   ├── parser.rs          # AST parsing with tree-sitter integration
│   └── walker.rs          # File system traversal with gitignore support
└── output/                 # Output formatting and generation
    ├── mod.rs             # OutputManager and routing logic
    ├── terminal.rs        # Pretty table formatting for console
    └── json.rs            # JSON serialization for exports

tests/
├── integration_tests.rs   # End-to-end CLI testing
└── integration/           # Additional integration test modules
    └── language_detection_test.rs
```

## Key Architecture Components

### AnalyzerEngine (src/analyzer/mod.rs)
- Central orchestrator for code analysis
- Manages language detection, parsing, and metrics collection
- Contains AnalysisStats for aggregating results

### CLI Layer (src/cli.rs)
- Uses clap derive API for argument parsing
- Defines CliArgs struct with validation
- Enums: SortBy, OutputFormat for type safety

### Error Handling (src/error.rs)
- AnalyzerError enum with specific error types
- Implements From traits for automatic conversion
- Provides Result<T> type alias

### Output System (src/output/)
- Dual output: terminal tables + JSON export
- OutputManager routes based on format selection
- Extensible design for adding new output formats

## Module Dependencies
- lib.rs → orchestrates all modules
- analyzer/ → independent analysis engine
- output/ → depends on analyzer results
- cli.rs → standalone argument parsing
- main.rs → minimal, uses lib.rs API