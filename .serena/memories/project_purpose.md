# Project Purpose

**Code Analyzer** is a CLI tool written in Rust that analyzes codebases to identify refactoring candidates using Abstract Syntax Tree (AST) parsing.

## Main Features

- **AST-based Code Analysis**: Uses tree-sitter to parse and analyze source code accurately
- **Multi-language Support**: Supports Rust, JavaScript, Python, Java, C, C++, Go, and TypeScript
- **Metrics Collection**: Counts lines of code, functions, classes, and calculates complexity scores
- **Gitignore Integration**: Respects .gitignore rules to focus on relevant source files
- **Dual Output**: Provides both terminal table output and JSON export for CI/CD integration
- **Parallel Processing**: Uses Rayon for efficient parallel file processing
- **Progress Reporting**: Shows progress during analysis with indicatif

## Target Users

- **Senior Developers/Architects**: Professionals who care about long-term code health
- **Development Teams**: For identifying refactoring priorities and technical debt
- **CI/CD Integration**: JSON output enables automated code quality monitoring

## Installation

- **Development**: `cargo install --path .`
- **Production**: Will be published to `crates.io` as `cargo install code-analyzer`