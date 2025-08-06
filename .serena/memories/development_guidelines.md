# Development Guidelines

## Design Patterns and Principles

### CLI Design Philosophy
- **Convention over Configuration**: Simple execution by default, optional flags for customization
- **Progressive Disclosure**: Basic usage is simple, advanced features available via flags
- **Fail Fast**: Clear error messages with helpful suggestions

### Architecture Principles
- **Separation of Concerns**: Clear module boundaries (CLI, analysis, output)
- **Single Responsibility**: Each module has a focused purpose
- **Composition over Inheritance**: Use traits and composition patterns
- **Error Transparency**: Meaningful error messages with context

### Performance Considerations
- **Parallel Processing**: Use `rayon` for CPU-intensive operations
- **Memory Efficiency**: Stream processing for large files
- **Progress Feedback**: Use `indicatif` for long-running operations
- **Lazy Evaluation**: Only process what's needed

### Code Quality Standards
- **Type Safety**: Leverage Rust's type system for correctness
- **Documentation**: All public APIs must be documented
- **Testing**: Both unit and integration tests required
- **Error Handling**: Comprehensive error handling with custom error types

## Tree-sitter Integration Patterns
- Language-specific parsers in separate modules
- Consistent AST traversal patterns
- Error handling for parse failures
- Language detection based on file extensions

## Output Design
- **Terminal Output**: Human-readable tables with `prettytable-rs`
- **JSON Output**: Machine-readable for CI/CD integration
- **Filtering and Sorting**: Comprehensive options for result refinement
- **Concurrent Generation**: Both outputs generated simultaneously

## Maintenance Guidelines
- Keep dependencies up to date
- Monitor for security advisories
- Maintain compatibility with stable Rust
- Follow semantic versioning for releases
- Regular performance benchmarking