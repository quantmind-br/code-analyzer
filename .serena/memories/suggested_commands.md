# Suggested Commands - Code Analyzer

## Makefile Commands (Preferred)

### Development Workflow
```bash
make build      # Development build
make release    # Optimized release build
make test       # Run all tests
make lint       # Run clippy with -D warnings
make fmt        # Format code
make quality    # Full quality check (fmt + lint + test)
make install    # Install to ~/.local/bin
make clean      # Clean build artifacts
make help       # Show all available targets
```

## Cargo Commands (Direct)

### Building & Installation
```bash
cargo build              # Development build
cargo build --release    # Release build (optimized)
cargo install --path .   # Install locally from source
```

### Testing
```bash
cargo test                              # Run all tests
cargo test --test integration_tests     # Integration tests only
cargo test -- --nocapture               # With verbose output
cargo test --lib                        # Library tests only
```

### Code Quality
```bash
cargo fmt                  # Format code
cargo fmt --check          # Check formatting without changes
cargo clippy               # Run clippy linter
cargo clippy -- -D warnings # Clippy with all warnings as errors
```

## Running the Tool

### Basic Usage
```bash
code-analyzer .                    # Analyze current directory
code-analyzer /path/to/project     # Analyze specific directory
code-analyzer --verbose            # With progress bar
```

### Filtering Options
```bash
code-analyzer --min-lines 100              # Files with 100+ lines
code-analyzer --min-functions 5            # Files with 5+ functions
code-analyzer --languages rust,python      # Specific languages only
code-analyzer --exclude "*.test.js"        # Exclude patterns
```

### Output Options
```bash
code-analyzer --output json                # JSON output only
code-analyzer --output both                # Terminal + JSON
code-analyzer --output-file report.json    # Custom output file
code-analyzer --compact                    # Minimal CI/CD output
code-analyzer --sort complexity            # Sort by complexity
code-analyzer --limit 20                   # Top 20 results
```

### CI Mode
```bash
code-analyzer --ci                         # CI mode (exit 2 if issues)
code-analyzer --ci --ci-max-candidates 5   # Allow up to 5 candidates
code-analyzer --only-changed-since HEAD~1  # Only changed files
```

### Threshold Configuration
```bash
code-analyzer --max-complexity-score 10.0  # Complexity threshold
code-analyzer --max-cc 15                  # Cyclomatic complexity
code-analyzer --max-loc 500                # Lines of code threshold
code-analyzer --max-functions-per-file 25  # Functions per file
```

## Quick Development Cycle
```bash
make check       # Fast syntax check (cargo check)
make test        # Run tests
make quality     # Full pre-commit check
```
