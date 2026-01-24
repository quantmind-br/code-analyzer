# Code Structure & Architecture - Code Analyzer

## Project Layout
```
src/
├── main.rs                 # Entry point - minimal (~80 lines), delegates to lib
├── lib.rs                  # Main library with public API and analysis orchestration
├── cli.rs                  # CLI argument parsing with clap derive API
├── error.rs                # Centralized error handling with AnalyzerError enum
├── analyzer/               # Core analysis engine
│   ├── mod.rs             # AnalyzerEngine orchestration, parallel analysis
│   ├── language.rs        # LanguageManager, SupportedLanguage enum, node mappings
│   ├── parser.rs          # AST parsing with tree-sitter (1370 lines - complexity hotspot)
│   ├── walker.rs          # File system traversal with gitignore support
│   └── git.rs             # Git integration for --only-changed-since feature
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
- Parallel file processing via rayon with thread-local parsers
- Contains AnalysisStats for aggregating results

### CLI Layer (src/cli.rs)
- Uses clap derive API for argument parsing
- Defines CliArgs struct with validation
- Enums: SortBy, OutputFormat, ColorMode for type safety
- CI mode flags: `--ci`, `--ci-max-candidates`

### Error Handling (src/error.rs)
- AnalyzerError enum with specific error types
- Implements From traits for automatic conversion
- Provides Result<T> type alias

### Output System (src/output/)
- Dual output: terminal tables + JSON export
- OutputManager routes based on format selection
- Compact mode for CI/CD pipelines

### Git Integration (src/analyzer/git.rs)
- `--only-changed-since` flag for incremental analysis
- Analyzes only files changed since a specific commit

## Module Dependencies
- lib.rs → orchestrates all modules
- analyzer/ → independent analysis engine
- output/ → depends on analyzer results
- cli.rs → standalone argument parsing
- main.rs → minimal, uses lib.rs API (thin binary pattern)

## Complexity Hotspots
- **parser.rs** (1370 lines) - Contains JSX sanitization state machine
- **language.rs** - Large match statements for 8 languages
- **walker.rs** - Known tech debt: excessive .unwrap() usage
