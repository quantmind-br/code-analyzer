# CODE-ANALYZER KNOWLEDGE BASE

**Generated:** 2026-01-24
**Commit:** 7d944b9
**Branch:** main

## OVERVIEW

Rust CLI/library for identifying refactoring candidates via tree-sitter AST parsing. Supports 8 languages (Rust, JS, TS, Python, Java, C, C++, Go). Dual output: terminal tables + JSON.

## STRUCTURE

```
code-analyzer/
├── src/
│   ├── main.rs           # CLI entry, CI mode handler
│   ├── lib.rs            # Library API, orchestration
│   ├── cli.rs            # clap derive args
│   ├── error.rs          # AnalyzerError enum
│   ├── analyzer/         # Core engine (see analyzer/AGENTS.md)
│   └── output/           # Terminal + JSON formatters
├── tests/                # Integration tests
└── history/              # Legacy planning docs
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add language support | `src/analyzer/language.rs` | Add grammar to Cargo.toml too |
| Modify complexity calc | `src/analyzer/parser.rs:971` | `calculate_cyclomatic_complexity` |
| Add CLI flag | `src/cli.rs` | clap derive, add to CliArgs |
| Change table output | `src/output/terminal.rs` | prettytable-rs |
| Change JSON schema | `src/output/json.rs` | Update serde structs |
| Add new metric | `src/analyzer/parser.rs` | Update FileAnalysis struct |
| Fix file traversal | `src/analyzer/walker.rs` | Uses `ignore` crate |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `AnalyzerEngine` | struct | `analyzer/mod.rs` | Main orchestrator |
| `run_analysis` | fn | `lib.rs:82` | Entry point for programmatic use |
| `FileParser` | struct | `analyzer/parser.rs` | AST parsing per file |
| `LanguageManager` | struct | `analyzer/language.rs` | Grammar loading, lang detection |
| `FileWalker` | struct | `analyzer/walker.rs` | Parallel file discovery |
| `OutputManager` | struct | `output/mod.rs` | Routes terminal/JSON output |
| `CliArgs` | struct | `cli.rs` | All CLI flags |
| `AnalyzerError` | enum | `error.rs` | Central error type |

## CONVENTIONS

### Quality Gates (MANDATORY before commit)
```bash
make quality  # or: cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### CLI Design
- All output commands MUST support `--json` flag
- CI mode: `--ci` exits 2 if candidates exceed `--ci-max-candidates`

### Testing
- Unit tests: co-located in `mod tests` blocks
- Integration: `tests/integration_tests.rs`
- Use `tempfile` crate for ephemeral test projects

## ANTI-PATTERNS

| Pattern | Why Forbidden |
|---------|---------------|
| Skip `make quality` | Zero tolerance for warnings |
| Excessive `.unwrap()` | Use `AnalyzerError`. Known debt in walker.rs |

## UNIQUE STYLES

- **Thin binary**: `main.rs` is ~80 lines. All logic in `lib.rs`.
- **Traditional mod.rs**: Uses `analyzer/mod.rs` pattern (not modern Rust 2018 style).
- **TSX parsing quirk**: JSX text requires `&` → `&amp;` before parsing. See `escape_ampersands_in_jsx_text`.

## COMMANDS

```bash
# Development
make build              # cargo build
make release            # cargo build --release
make test               # cargo test
make lint               # cargo clippy -- -D warnings
make quality            # fmt + lint + test (pre-commit)

# Install
make install            # → ~/.local/bin/code-analyzer

# Run
code-analyzer .                    # analyze current dir
code-analyzer --output json        # JSON only
code-analyzer --ci --ci-max-candidates 5  # CI mode
```

## NOTES

- **Exit codes**: 0=success, 1=error, 2=CI threshold exceeded
- **Parallelism**: Uses rayon for file analysis. Thread-local parsers.
- **Progress**: `--verbose` shows indicatif progress bar
- **File size limit**: Default 10MB, configurable via `--max-file-size-mb`
- **Complexity hotspot**: `src/analyzer/parser.rs` (1370 lines) - consider splitting
