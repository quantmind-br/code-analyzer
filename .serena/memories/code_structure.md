# Code Structure

## Project Layout

```
code-analyzer/
├── src/
│   ├── main.rs           # Entry point, CLI argument parsing
│   ├── lib.rs            # Main analysis logic and coordination 
│   ├── cli.rs            # CLI argument definitions and types
│   ├── error.rs          # Error handling and custom error types
│   ├── analyzer/         # AST analysis modules
│   └── output/           # Output formatting modules
├── tests/
│   └── integration_tests.rs  # Integration tests
├── PRPs/                 # Project Requirements and Planning
│   ├── ai_docs/         # AI-generated documentation
│   ├── scripts/         # Helper scripts (e.g., prp_runner.py)
│   └── templates/       # Document templates
├── target/              # Rust build artifacts
├── Cargo.toml           # Rust package configuration
├── Cargo.lock           # Dependency lock file
├── PLAN.md              # Project overview in Portuguese
├── refactor-candidates.json  # Example output
└── test-results.json    # Test analysis results
```

## Module Structure

- **main.rs**: Simple entry point that parses CLI args and delegates to lib
- **cli.rs**: Comprehensive CLI argument definitions using clap derive API
- **lib.rs**: Core analysis orchestration, file discovery, and result aggregation
- **error.rs**: Custom error types and error handling utilities
- **analyzer/**: AST parsing and metric calculation modules (per language)
- **output/**: Terminal table and JSON output formatting

## Key Features in Code

- Uses `tree-sitter` for accurate AST-based analysis
- Parallel processing with `rayon` for performance
- Progress reporting during analysis
- Comprehensive filtering and sorting options
- Multiple output formats (table, JSON)