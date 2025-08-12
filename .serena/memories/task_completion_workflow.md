# Task Completion Workflow - Code Analyzer

## When Task is Completed - Required Checks

### 1. Code Quality Verification
Always run these commands after making changes:

```bash
# Format check
cargo fmt --check

# Linting with strict warnings
cargo clippy -- -D warnings

# All tests must pass
cargo test
```

### 2. Integration Testing
Run the full test suite including integration tests:

```bash
# Run all tests including integration
cargo test

# Specifically test CLI integration
cargo test --test integration_tests

# Test with verbose output if needed
cargo test -- --nocapture
```

### 3. Build Verification
Ensure the project builds in both modes:

```bash
# Debug build
cargo build

# Release build (catches optimization issues)
cargo build --release
```

### 4. Documentation Updates
If public APIs changed:

```bash
# Generate documentation
cargo doc

# Check for documentation warnings
cargo doc --open
```

## Development Workflow

### Before Starting Work
1. `git status` - Check current state
2. `cargo test` - Ensure tests pass
3. `git pull origin main` - Get latest changes

### During Development
1. `cargo check` - Fast syntax validation
2. `cargo test --lib` - Quick library tests
3. `cargo clippy` - Regular linting checks

### Before Committing
1. **MUST RUN**: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`
2. If all pass, then commit
3. Never commit if any of these fail

## Common Issues & Solutions

### Tree-sitter Language Parsing
- If new language support added, ensure grammar dependencies are in Cargo.toml
- Test with sample files in `test_mixed_languages/`

### Performance Testing
- Use `--release` build for performance testing
- Test with large codebases using `cargo run --release`

### Windows Compatibility
- Ensure path handling works with Windows paths (`\` vs `/`)
- Test CLI commands with Windows command prompt