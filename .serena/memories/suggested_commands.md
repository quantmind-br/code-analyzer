# Suggested Commands for Development

## Build and Run Commands

### Development Build
```cmd
cargo build
```

### Release Build (Optimized)
```cmd
cargo build --release
```

### Run in Development
```cmd
cargo run
cargo run -- --help
cargo run -- path/to/analyze --verbose
```

### Install Locally for Testing
```cmd
cargo install --path .
```

## Testing Commands

### Run All Tests
```cmd
cargo test
```

### Run Integration Tests
```cmd
cargo test --test integration_tests
```

### Test with Verbose Output
```cmd
cargo test -- --nocapture
```

## Code Quality Commands

### Format Code
```cmd
cargo fmt
```

### Check Formatting (CI)
```cmd
cargo fmt --check
```

### Lint with Clippy
```cmd
cargo clippy
```

### Clippy for CI (Fail on Warnings)
```cmd
cargo clippy -- -D warnings
```

### Check Without Building
```cmd
cargo check
```

## Analysis and Documentation

### Generate Documentation
```cmd
cargo doc
cargo doc --open
```

### Security Audit
```cmd
cargo audit
```

## Windows-Specific Utilities

### List Directory Contents
```cmd
dir
```

### Find Files
```cmd
where filename
findstr /s "pattern" *.rs
```

### Environment Variables
```cmd
echo %PATH%
set RUST_LOG=debug
```

### Git Operations
```cmd
git status
git add .
git commit -m "message"
git log --oneline
```