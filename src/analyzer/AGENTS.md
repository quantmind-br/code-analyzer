# ANALYZER MODULE

Core analysis engine. AST parsing, file discovery, language detection.

## STRUCTURE

```
analyzer/
├── mod.rs       # AnalyzerEngine orchestration, parallel analysis
├── parser.rs    # FileParser, AST traversal, complexity scoring (1370 lines)
├── language.rs  # LanguageManager, SupportedLanguage enum, node mappings
├── walker.rs    # FileWalker, gitignore-aware traversal
└── git.rs       # Git integration for --only-changed-since
```

## WHERE TO LOOK

| Task | File | Function/Struct |
|------|------|-----------------|
| Add new language | `language.rs` | `SupportedLanguage` enum, add grammar |
| Modify complexity formula | `parser.rs:971` | `calculate_cyclomatic_complexity` |
| Add AST metric | `parser.rs` | Update `FileAnalysis`, add counting fn |
| Change file filtering | `walker.rs` | `FilterConfig`, `FileWalker::discover_files` |
| Git diff analysis | `git.rs` | `get_changed_files` |

## KEY TYPES

| Type | Purpose |
|------|---------|
| `AnalyzerEngine` | Orchestrates analysis pipeline |
| `FileParser` | Parses single file via tree-sitter |
| `FileAnalysis` | Per-file metrics (LOC, functions, complexity) |
| `LanguageManager` | Maps extensions → grammars |
| `SupportedLanguage` | Enum of 8 supported languages |
| `RefactoringThresholds` | Configurable limits for CI mode |

## CONVENTIONS

### Adding a Language
1. Add variant to `SupportedLanguage` enum
2. Implement `control_flow_node_kinds()`, `function_node_kinds()`, etc.
3. Add tree-sitter grammar to `Cargo.toml`
4. Add extension mapping in `LanguageManager`

### AST Traversal
- Use `count_nodes_iterative` (stack-safe) NOT recursion
- Use `TreeCursor` for manual traversal
- Thread-local parsers for parallelism

## ANTI-PATTERNS

| Pattern | Why |
|---------|-----|
| Recursive AST walk | Stack overflow risk. Use `TreeCursor`. |
| Shared parser across threads | Not thread-safe. Clone `LanguageManager`. |
| Raw `&` in TSX text | Breaks parser. Use `escape_ampersands_in_jsx_text`. |

## COMPLEXITY HOTSPOTS

- **parser.rs:566** - `escape_ampersands_in_jsx_text` (state machine, 4+ nesting levels)
- **language.rs:195** - Large `match` for `control_flow_node_kinds` (8 languages)
- **walker.rs** - 30+ `.unwrap()` calls (known tech debt)

## NOTES

- Cyclomatic complexity: McCabe's formula via `count_control_flow` + `count_logical_operators`
- `FileAnalysisResult` wraps `FileAnalysis` + parse warnings
- Parallelism via `rayon::par_iter()` + `Arc<Mutex<Vec<T>>>`
