# GitHub Copilot Instructions for Code Analyzer

## Project Overview

**Code Analyzer** is a CLI tool written in Rust for analyzing codebases to identify refactoring candidates using AST parsing. It supports multiple languages (Rust, JavaScript, Python, Java, C, C++, Go, TypeScript) and provides both terminal and JSON output for CI/CD integration.

## Tech Stack

- **Language**: Rust
- **AST Parsing**: tree-sitter + language grammars
- **CLI Framework**: clap (derive API)
- **Parallelism**: rayon
- **Testing**: Rust standard testing + integration tests

## Coding Guidelines

### Testing
- Always write tests for new features
- Run `cargo test` before committing
- Use `tempfile` for creating test projects
- Integration tests in `tests/integration_tests.rs`

### Code Style
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` before committing
- Follow existing patterns in `src/analyzer/`
- Add `--json` flag support to all output commands

### Git Workflow
- Always commit `.beads/issues.jsonl` with code changes
- Run `bd sync` at end of work sessions
- Run full quality check: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`

## Issue Tracking with bd

**CRITICAL**: This project uses **bd** for ALL task tracking. Do NOT create markdown TODO lists.

### Essential Commands

```bash
# Find work
bd ready --json                    # Unblocked issues
bd stale --days 30 --json          # Forgotten issues

# Create and manage
bd create "Title" -t bug|feature|task -p 0-4 --json
bd create "Subtask" --parent <epic-id> --json  # Hierarchical subtask
bd update <id> --status in_progress --json
bd close <id> --reason "Done" --json

# Search
bd list --status open --priority 1 --json
bd show <id> --json

# Sync (CRITICAL at end of session!)
bd sync  # Force immediate export/commit/push
```

### Workflow

1. **Check ready work**: `bd ready --json`
2. **Claim task**: `bd update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** `bd create "Found bug" -p 1 --deps discovered-from:<parent-id> --json`
5. **Complete**: `bd close <id> --reason "Done" --json`
6. **Sync**: `bd sync` (flushes changes to git immediately)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

## Project Structure

```
code-analyzer/
├── src/
│   ├── lib.rs           # Main library API
│   ├── main.rs          # CLI entry point
│   ├── cli.rs           # CLI argument parsing
│   ├── error.rs         # Error handling
│   ├── analyzer/        # Core analysis engine
│   │   ├── mod.rs       # AnalyzerEngine
│   │   ├── language.rs  # Language detection
│   │   ├── parser.rs    # AST parsing
│   │   └── walker.rs    # File system traversal
│   └── output/          # Output formatting
│       ├── mod.rs
│       ├── terminal.rs  # Pretty table output
│       └── json.rs      # JSON output
├── tests/               # Integration tests
├── .beads/
│   ├── beads.db         # SQLite database (DO NOT COMMIT)
│   └── issues.jsonl     # Git-synced issue storage
└── Cargo.toml
```

## CLI Help

Run `bd <command> --help` to see all available flags for any command.
For example: `bd create --help` shows `--parent`, `--deps`, `--assignee`, etc.

## Important Rules

- Use bd for ALL task tracking
- Always use `--json` flag for programmatic use
- Run `bd sync` at end of sessions
- Run quality checks before committing
- Run `bd <cmd> --help` to discover available flags
- Do NOT create markdown TODO lists
- Do NOT commit `.beads/beads.db` (JSONL only)

---

**For detailed workflows and advanced features, see [AGENTS.md](../AGENTS.md)**
