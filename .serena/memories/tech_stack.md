# Tech Stack

## Language
- **Rust 2021 Edition**

## Core Dependencies

### CLI and Argument Parsing
- **clap 4.0** - CLI argument parsing with derive API

### AST Parsing and Code Analysis
- **tree-sitter 0.20** - Core AST parsing library
- **tree-sitter language grammars**:
  - tree-sitter-rust 0.20
  - tree-sitter-javascript 0.20  
  - tree-sitter-python 0.20
  - tree-sitter-java 0.20
  - tree-sitter-c 0.20
  - tree-sitter-cpp 0.20
  - tree-sitter-go 0.20
  - tree-sitter-typescript 0.20

### File System and Directory Traversal
- **ignore 0.4** - Directory traversal with gitignore support

### Output and Formatting
- **prettytable-rs 0.10** - Terminal table formatting
- **serde 1.0** - JSON serialization with derive features
- **serde_json 1.0** - JSON handling
- **chrono 0.4** - Datetime handling with serde features

### Performance and Progress
- **rayon 1.8** - Parallel processing
- **indicatif 0.17** - Progress reporting and bars

## Development Dependencies
- **tempfile 3.0** - Temporary files for testing
- **assert_cmd 2.0** - CLI testing utilities
- **predicates 3.0** - Test assertions

## Build Configuration
- **Release Profile**: LTO enabled, single codegen unit, panic=abort for optimization