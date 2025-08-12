# Tech Stack - Code Analyzer

## Core Language & Runtime
- **Rust 2021 Edition**: Modern Rust with latest features
- **Minimum Rust Version**: 1.70+

## Key Dependencies

### CLI & User Interface
- **clap 4.0**: Command-line argument parsing with derive API
- **prettytable-rs 0.10**: Terminal table formatting
- **indicatif 0.17**: Progress bars and spinners

### AST Parsing & Language Support
- **tree-sitter 0.20**: Core AST parsing engine
- **Language Grammars**: Individual tree-sitter parsers for:
  - tree-sitter-rust, tree-sitter-javascript, tree-sitter-python
  - tree-sitter-java, tree-sitter-c, tree-sitter-cpp
  - tree-sitter-go, tree-sitter-typescript

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

## Architecture Pattern
- Modular design with clear separation:
  - `analyzer/` - Core analysis engine with language detection, parsing, and walking
  - `output/` - Dual output system (terminal + JSON)
  - `cli.rs` - Command-line interface and argument handling
  - `error.rs` - Centralized error handling