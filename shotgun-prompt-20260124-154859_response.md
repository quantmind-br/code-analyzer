# Code Quality Refactoring Plan

**Analyzed**: 2026-01-24
**Status**: Reviewed and prioritized

## Implementation Order

Execute in this sequence for optimal risk/value tradeoff:

---

## Phase 1: Quick Wins (Low Risk, High Value)

### 1. Eliminate Duplicate Execution Logic in Library Entry Point

**ID**: cq-002 | **Effort**: Small | **Priority**: High

**Problem**: `run_analysis` and `run_analysis_returning_report` in `src/lib.rs` share ~90% identical code (lines 82-129 and 135-182).

**Solution**: Extract common workflow into private helper.

```rust
// src/lib.rs

fn execute_analysis_core(args: &CliArgs) -> Result<AnalysisReport> {
    args.validate()?;
    
    if args.verbose {
        println!("Starting code analysis...");
        println!("Target: {}", args.target_path().display());
        if !args.languages.is_empty() {
            println!("Languages: {}", args.languages.join(", "));
        }
        println!("Min lines: {}", args.min_lines);
        if let Some(max_lines) = args.max_lines {
            println!("Max lines: {max_lines}");
        }
        println!("Output format: {}", args.output);
        println!();
    }
    
    let mut analyzer = AnalyzerEngine::from_cli_args(args)?;
    analyzer.analyze_project(args.target_path(), args)
}

pub fn run_analysis(args: CliArgs) -> Result<()> {
    let report = execute_analysis_core(&args)?;
    // ... output handling ...
    Ok(())
}

pub fn run_analysis_returning_report(args: CliArgs) -> Result<AnalysisReport> {
    let report = execute_analysis_core(&args)?;
    // ... output handling ...
    Ok(report)
}
```

**Affected Files**: `src/lib.rs`

---

### 2. Improve Panic Messages for Parallel Execution

**ID**: cq-005 (adjusted) | **Effort**: Small | **Priority**: Low

**Problem**: `.unwrap()` on Mutex locks provides no context on panic.

**Solution**: Replace with `.expect()` for better debugging. Do NOT add error propagation — mutex poison is not recoverable.

```rust
// src/analyzer/walker.rs (line 148, 172, 185, 205-209)

// Before
files.lock().unwrap()
Arc::try_unwrap(files).unwrap().into_inner().unwrap()

// After
files.lock().expect("walker mutex poisoned - worker thread panicked")
Arc::try_unwrap(files)
    .expect("Arc still has multiple owners after parallel walk completed")
    .into_inner()
    .expect("walker mutex poisoned")
```

**Affected Files**: `src/analyzer/walker.rs`, `src/analyzer/mod.rs`

---

## Phase 2: Targeted Refactoring (Medium Risk)

### 3. Simplify JSX Sanitizer State Machine

**ID**: cq-004 | **Effort**: Small-Medium | **Priority**: Medium

**Problem**: `escape_ampersands_in_jsx_text` (lines 575-800+ in parser.rs) is a 200+ line function with 4+ nesting levels and inline helper functions.

**Prerequisites**:
- [ ] Add unit tests for edge cases BEFORE refactoring:
  - Unterminated JSX tags
  - Nested `{}` expressions
  - Multiple consecutive `&` characters
  - Already-escaped entities (`&amp;`, `&#123;`)
  - Mixed JSX text and expressions

**Solution**: Extract to standalone struct.

```rust
// src/analyzer/jsx_sanitizer.rs (new file)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Normal,
    InTag,
    InText,
    InExpr,
}

pub struct JsxSanitizer {
    state: State,
    jsx_depth: usize,
    // ... other fields from current function
}

impl JsxSanitizer {
    pub fn new() -> Self { ... }
    
    pub fn process(&mut self, source: &str) -> String { ... }
    
    fn handle_normal(&mut self, ch: char) -> Option<char> { ... }
    fn handle_in_tag(&mut self, ch: char) -> Option<char> { ... }
    fn handle_in_text(&mut self, ch: char) -> Option<char> { ... }
    fn handle_in_expr(&mut self, ch: char) -> Option<char> { ... }
}

// Helper functions become associated functions
impl JsxSanitizer {
    fn is_ident_char(ch: char) -> bool { ... }
    fn looks_like_entity(chars: &[char], i: usize) -> bool { ... }
    // ...
}
```

**Affected Files**: 
- `src/analyzer/jsx_sanitizer.rs` (new)
- `src/analyzer/parser.rs` (remove function, add `use`)
- `src/analyzer/mod.rs` (add module)

---

## Phase 3: Structural Refactoring (Higher Risk)

### 4. Decompose Monolithic FileParser Module

**ID**: cq-001 (refined) | **Effort**: Medium | **Priority**: Medium

**Problem**: `parser.rs` (1417 lines) mixes data structures, parsing logic, metrics calculation, and sanitization.

**Prerequisites**:
- [ ] Complete cq-004 (JSX sanitizer already extracted)
- [ ] Add integration tests covering FileParser public API
- [ ] Ensure `make quality` passes

**Solution**: Split into 4 modules under `src/analyzer/parser/`:

```
src/analyzer/parser/
├── mod.rs          # FileParser struct, orchestration, re-exports
├── types.rs        # FileAnalysis, AnalysisReport, ProjectSummary, etc.
├── metrics.rs      # calculate_cyclomatic_complexity, count_* functions
└── (jsx_sanitizer.rs already extracted in cq-004)
```

**Migration Steps**:
1. Create `parser/` directory
2. Move data structs (lines 1-120) to `types.rs`
3. Move metric functions (lines 900-1100) to `metrics.rs`
4. Keep `FileParser` impl in `mod.rs`
5. Update all imports across codebase
6. Run `make quality`

**Affected Files**:
- `src/analyzer/parser.rs` → split into module
- All files importing from `analyzer::parser`

---

## Deferred

### Refactor Language Support to Trait-Based Registry

**ID**: cq-003 | **Status**: Deferred

**Reason**: The current enum + match pattern with 9 languages is idiomatic Rust. The compiler enforces exhaustive updates via match exhaustiveness. A trait-based approach:
- Loses compile-time exhaustiveness guarantees
- Adds runtime indirection
- Increases file count without proportional benefit

**Reconsider When**: >15 languages are supported, or if language definitions need runtime loading (plugins).

**Alternative (Optional)**: If match block size becomes unwieldy, consider a declarative macro:

```rust
language_spec! {
    Rust => {
        function_kinds: ["function_item"],
        class_kinds: ["struct_item", "enum_item", "impl_item"],
        control_flow_kinds: ["if_expression", "match_arm", ...],
    },
    JavaScript => { ... },
}
```

---

## Summary

| Phase | ID | Action | Effort | Risk |
|-------|-----|--------|--------|------|
| 1 | cq-002 | DRY fix in lib.rs | Small | Low |
| 1 | cq-005 | `.expect()` instead of `.unwrap()` | Small | Low |
| 2 | cq-004 | Extract JSX sanitizer | Small-Medium | Medium |
| 3 | cq-001 | Decompose parser.rs | Medium | Medium |
| — | cq-003 | **Deferred** | — | — |

**Total Estimated Effort**: ~2-3 focused sessions

**Quality Gates**: Run `make quality` after each phase before proceeding.
