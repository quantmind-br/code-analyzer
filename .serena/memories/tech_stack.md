# Tech Stack - Code Analyzer

## Core Language & Runtime
- **Rust 2021 Edition**: Modern Rust with latest features
- **Minimum Rust Version**: 1.70+

## Key Dependencies

### CLI & User Interface
- **clap 4.0**: Command-line argument parsing with derive API
- **prettytable-rs 0.10**: Terminal table formatting
- **indicatif 0.17**: Progress bars and spinners
- **atty 0.2**: TTY detection for color output

### AST Parsing & Language Support
- **tree-sitter 0.26**: Core AST parsing engine
- **Language Grammars**: Individual tree-sitter parsers for:
  - tree-sitter-rust 0.24
  - tree-sitter-javascript 0.25
  - tree-sitter-python 0.25
  - tree-sitter-java 0.23
  - tree-sitter-c 0.24
  - tree-sitter-cpp 0.23
  - tree-sitter-go 0.25
  - tree-sitter-typescript 0.23

### File System & Data Processing
- **ignore 0.4**: Directory traversal with gitignore support
- **rayon 1.8**: Parallel processing for performance
- **serde 1.0 + serde_json 1.0**: JSON serialization
- **chrono 0.4**: Date/time handling for reports

### Development & Testing
- **tempfile 3.0**: Temporary directories for tests
- **assert_cmd 2.0**: CLI testing framework
- **predicates 3.0**: Test assertion predicates

## Build Features
- **Default**: `full-analysis` feature enabled
- **Release Profile**: LTO optimization, single codegen unit, abort on panic

## Architecture Patterns
- **Thin binary**: main.rs is ~80 lines, all logic in lib.rs
- **Traditional mod.rs**: Uses analyzer/mod.rs pattern
- **Thread-local parsers**: For safe parallel processing with rayon
- **Dual crate**: Both library and binary in one package
