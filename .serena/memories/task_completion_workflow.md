# Task Completion Workflow - Code Analyzer

## When Task is Completed - Required Checks

### 1. Code Quality Verification (MANDATORY)
Use the Makefile for consistent quality checks:

```bash
# Full quality check (REQUIRED before commit)
make quality

# Or run individually:
cargo fmt --check          # Format check
cargo clippy -- -D warnings # Linting with strict warnings
cargo test                  # All tests must pass
```

### 2. Integration Testing
Run the full test suite including integration tests:

```bash
# Run all tests
make test

# Or specifically:
cargo test --test integration_tests

# Test with verbose output if needed
cargo test -- --nocapture
```

### 3. Build Verification
Ensure the project builds in both modes:

```bash
make build      # Debug build
make release    # Release build (catches optimization issues)
```

### 4. Documentation Updates
If public APIs changed:

```bash
cargo doc --open  # Generate and view documentation
```

## Development Workflow

### Before Starting Work
1. `git status` - Check current state
2. `make test` - Ensure tests pass
3. `git pull origin main` - Get latest changes

### During Development
1. `make check` - Fast syntax validation
2. `cargo test --lib` - Quick library tests
3. `make lint` - Regular linting checks

### Before Committing
1. **MUST RUN**: `make quality` (runs fmt check + clippy + tests)
2. If all pass, then commit
3. Never commit if any of these fail

## Makefile Targets Reference

| Target | Command | Purpose |
|--------|---------|---------|
| `make build` | `cargo build` | Development build |
| `make release` | `cargo build --release` | Optimized build |
| `make test` | `cargo test` | Run all tests |
| `make lint` | `cargo clippy -- -D warnings` | Strict linting |
| `make fmt` | `cargo fmt` | Format code |
| `make quality` | fmt + lint + test | Pre-commit check |
| `make install` | Install to ~/.local/bin | Local installation |
| `make clean` | `cargo clean` | Clean build artifacts |

## Common Issues & Solutions

### Tree-sitter Language Parsing
- If new language support added, ensure grammar dependencies are in Cargo.toml
- Test with sample files in `test_mixed_languages/`
- TSX/JSX quirk: `&` must be escaped as `&amp;` before parsing

### Performance Testing
- Use `make release` for performance testing
- Test with large codebases using `cargo run --release`

### Known Technical Debt
- `walker.rs` contains excessive `.unwrap()` calls - use `AnalyzerError` for new code
