# Suggested Commands - Code Analyzer

## Development Commands

### Building & Installation
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Install locally from source
cargo install --path .
```

### Testing
```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test integration_tests

# Run with verbose output
cargo test -- --nocapture

# Run specific test module
cargo test --test integration_tests -- language_detection
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting without making changes
cargo fmt --check

# Run clippy linter
cargo clippy

# Clippy with all warnings as errors (CI mode)
cargo clippy -- -D warnings

# Full quality check (suitable for CI)
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Running the Tool
```bash
# Basic analysis of current directory
cargo run

# Analyze specific directory
cargo run /path/to/project

# With filters and options
cargo run -- --min-lines 100 --sort complexity --output json

# Using installed binary
code-analyzer --help
code-analyzer --min-functions 5 --languages rust,python
```

## Windows-Specific Commands

### System Commands
- `dir` - List directory contents (instead of `ls`)
- `type file.txt` - Display file contents (instead of `cat`)
- `findstr "pattern" *.rs` - Search in files (instead of `grep`)
- `where code-analyzer` - Find executable location (instead of `which`)

### Git Operations
```bash
git status
git add .
git commit -m "message"
git push origin main
```

## Cargo Shortcuts
```bash
# Quick development cycle
cargo check          # Fast syntax check
cargo test --lib      # Test library code only
cargo doc --open      # Generate and open documentation
```