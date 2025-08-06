# Task Completion Workflow

## When a Task is Completed

After implementing any code changes, follow this checklist:

### 1. Code Quality Checks
```cmd
# Format the code
cargo fmt

# Run clippy for linting
cargo clippy

# Check clippy with no warnings allowed (CI standard)
cargo clippy -- -D warnings
```

### 2. Testing
```cmd
# Run all tests
cargo test

# Run integration tests specifically
cargo test --test integration_tests

# Test with verbose output if needed
cargo test -- --nocapture
```

### 3. Build Verification
```cmd
# Debug build
cargo check

# Release build to ensure optimization compatibility
cargo build --release
```

### 4. Documentation
```cmd
# Generate and check documentation
cargo doc

# Verify no documentation warnings
cargo doc --no-deps
```

### 5. Final Validation
- Ensure all tests pass
- No compiler warnings
- No clippy warnings
- Code is properly formatted
- Documentation is complete and accurate

## Pre-commit Standards
- All code must be formatted with `cargo fmt`
- All clippy warnings must be resolved
- All tests must pass
- No compiler warnings allowed in release build

## CI/CD Readiness
The project follows Rust best practices for CI/CD:
- Formatted code (`cargo fmt --check`)
- Linted code (`cargo clippy -- -D warnings`)
- Passing tests (`cargo test`)
- Successful release build (`cargo build --release`)