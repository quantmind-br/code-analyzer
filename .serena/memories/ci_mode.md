# CI Mode & Refactoring Thresholds

## CI Mode Overview

The tool includes a built-in CI mode for automated quality gates in pipelines.

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success - no issues or within threshold |
| 1 | Error - execution failed |
| 2 | CI threshold exceeded - too many refactoring candidates |

### Basic CI Usage
```bash
# Fail if ANY refactoring candidates found
code-analyzer --ci

# Allow up to N candidates before failing
code-analyzer --ci --ci-max-candidates 5
```

## Refactoring Thresholds

Files are flagged as refactoring candidates based on configurable thresholds:

| Threshold | Default | CLI Flag | Description |
|-----------|---------|----------|-------------|
| Complexity Score | 10.0 | `--max-complexity-score` | Composite complexity metric |
| Cyclomatic Complexity | 15 | `--max-cc` | McCabe's CC per NIST guidance |
| Lines of Code | 500 | `--max-loc` | File size threshold |
| Functions per File | 25 | `--max-functions-per-file` | Function count limit |

### Custom Thresholds
```bash
code-analyzer --ci \
  --max-complexity-score 8.0 \
  --max-cc 10 \
  --max-loc 300 \
  --max-functions-per-file 15
```

## Git Integration

Analyze only files changed since a specific commit:
```bash
# Files changed since last commit
code-analyzer --only-changed-since HEAD~1

# Files changed since main branch
code-analyzer --only-changed-since main

# Files changed since specific commit
code-analyzer --only-changed-since abc123
```

## CI Pipeline Examples

### GitHub Actions
```yaml
- name: Code Quality Check
  run: code-analyzer --ci --ci-max-candidates 0
```

### Pre-commit Hook
```bash
#!/bin/bash
code-analyzer --ci --only-changed-since HEAD~1
```

## RefactoringThresholds Struct

Located in `src/analyzer/parser.rs`:
- `from_cli()` - Creates thresholds from CLI args
- Uses defaults when CLI flags not provided
- `identify_refactoring_candidates()` - Applies thresholds to analysis results

## Implementation Details

- CI mode is handled in `src/main.rs` via `run_ci_mode()`
- Thresholds are passed to `identify_refactoring_candidates()`
- Results are printed with first 10 candidates shown
- Exit code 2 triggers when `candidates.len() > ci_max`
