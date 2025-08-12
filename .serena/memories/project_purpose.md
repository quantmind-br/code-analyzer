# Project Purpose - Code Analyzer

This is a CLI tool written in Rust for analyzing codebases to identify refactoring candidates using AST (Abstract Syntax Tree) parsing. The tool is designed to help developers and teams:

## Main Functionality
- **AST-based Analysis**: Uses tree-sitter for precise code parsing
- **Multi-language Support**: Supports Rust, JavaScript, Python, Java, C, C++, Go, and TypeScript
- **Detailed Metrics**: Counts lines of code, functions, classes, and calculates complexity scores
- **Gitignore Respect**: Automatically ignores files specified in .gitignore
- **Parallel Processing**: Optimized for performance with parallel analysis using rayon
- **Dual Output**: Both terminal tables and JSON export for CI/CD integration

## Target Use Cases
- **Refactoring Planning**: Identify complex files that need attention
- **Code Quality Monitoring**: Track metrics over time in CI/CD pipelines  
- **Technical Debt Analysis**: Quantify code health with objective metrics
- **Team Onboarding**: Help new developers understand codebase structure
- **Architecture Reviews**: Data-driven insights for code reviews

## Key Features
- Flexible filtering by lines, functions, classes, file size
- Intelligent sorting options
- Custom exclusion patterns beyond .gitignore
- Progress reporting for large codebases
- Language-specific analysis with tree-sitter grammars