# Rust Code Conventions and Style

## Naming Conventions
- **snake_case** for variables, functions, modules: `min_lines`, `run_analysis`
- **PascalCase** for structs and enums: `CliArgs`, `SortBy`, `OutputFormat`
- **SCREAMING_SNAKE_CASE** for constants
- **kebab-case** for CLI arguments: `--min-lines`, `--max-file-size-mb`

## Documentation Style
- Use `///` for public API documentation
- Document all public structs, enums, and functions
- Include examples in documentation where helpful
- Use `#[command(about = "...")]` for CLI help text

## Code Organization
- One main concept per file
- Group related functionality in modules
- Use `pub` sparingly, prefer module-level privacy
- Implement `Display` and `Debug` traits where appropriate

## Error Handling
- Use `Result<T, E>` for fallible operations
- Custom error types in `error.rs` module
- Use `?` operator for error propagation
- Provide meaningful error messages

## CLI Design Patterns
- Use clap's derive API with `#[derive(Parser)]`
- Use `#[arg()]` attributes for argument configuration
- Implement `ValueEnum` for choice-based arguments
- Default values in argument definitions

## Testing
- Integration tests in `tests/` directory
- Use `assert_cmd` for CLI testing
- Use `tempfile` for temporary test directories
- Test both success and error cases