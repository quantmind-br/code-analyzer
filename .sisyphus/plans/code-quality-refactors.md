# Code Quality Refactors

## Context

### Original Request
Implement 4 code quality improvements identified in shotgun analysis:
1. Deduplicate library entry points (cq-004)
2. Language configuration strategy pattern (cq-002)
3. Extract sanitizer module (cq-001)
4. Parser reuse in parallel analysis (cq-005)

### Interview Summary
**Key Discussions**:
- TDD approach selected: write/adjust tests first, then implement
- All 4 items in scope (including cq-005)
- One commit per completed refactor

**Research Findings**:
- `rayon 1.10.0` supports `map_init` - no dependency update needed
- Sanitizer functions are internal to `parser.rs` only
- Language.rs has 9 languages × 10 match blocks = shotgun surgery anti-pattern

### Metis Review
**Identified Gaps** (addressed):
- TDD clarification: characterization tests acceptable for behavior-preserving refactors
- LanguageSpec: must use `&'static` slices, zero allocation
- Sanitizer module: `pub(crate)` visibility, lives under `src/analyzer/`
- Guardrails: preserve all public API signatures and CLI/JSON output

---

## Work Objectives

### Core Objective
Refactor code-analyzer internals to improve maintainability without changing external behavior.

### Concrete Deliverables
- `src/lib.rs` with deduplicated entry points
- `src/analyzer/language.rs` with `LanguageSpec` strategy pattern
- `src/analyzer/sanitizer.rs` new module with extracted JSX/TS sanitization
- `src/analyzer/mod.rs` with `map_init` parser reuse

### Definition of Done
- [ ] `make quality` passes (fmt + clippy + test)
- [ ] All existing tests pass unchanged
- [ ] CLI output identical on test fixtures
- [ ] JSON schema unchanged

### Must Have
- TDD: write/adjust characterization test before each change
- One commit per refactor with descriptive message
- Preserve all public API signatures

### Must NOT Have (Guardrails)
- No changes to CLI argument parsing
- No changes to JSON output schema
- No new language support (save for separate task)
- No parser.rs changes beyond sanitizer extraction
- No dependency updates

---

## Verification Strategy (MANDATORY)

### Test Decision
- **Infrastructure exists**: YES (cargo test, make quality)
- **User wants tests**: TDD (characterization tests first)
- **Framework**: cargo test (built-in)

### TDD Workflow
Each TODO follows RED-GREEN-REFACTOR:
1. **RED**: Write or identify test that validates current behavior
2. **GREEN**: Refactor while keeping test passing
3. **VERIFY**: `make quality` passes

---

## Task Flow

```
Task 0 (baseline) → Task 1 (cq-004) → Task 2 (cq-002) → Task 3 (cq-001) → Task 4 (cq-005)
```

## Parallelization

| Task | Depends On | Reason |
|------|------------|--------|
| 0 | None | Baseline verification |
| 1 | 0 | Trivial, builds confidence |
| 2 | 1 | Foundational for cq-001 |
| 3 | 2 | Uses LanguageSpec.sanitizer |
| 4 | 3 | Independent but last for stability |

---

## TODOs

- [ ] 0. Baseline Verification

  **What to do**:
  - Run `make quality` to establish baseline
  - Run `cargo test -- --nocapture` to verify all tests pass
  - Capture current test count for comparison

  **Must NOT do**:
  - Make any code changes

  **Parallelizable**: NO (must complete first)

  **References**:
  - `Makefile` - quality target definition
  - `AGENTS.md:Quality Gates` - expected commands

  **Acceptance Criteria**:
  - [ ] `make quality` → 0 errors, 0 warnings
  - [ ] Record test count: `cargo test 2>&1 | grep "test result"`

  **Commit**: NO

---

- [x] 1. [cq-004] Deduplicate Library Entry Points

  **What to do**:
  1. Identify existing test that exercises `run_analysis` (characterization)
  2. Refactor `run_analysis` to call `run_analysis_returning_report(args).map(|_| ())`
  3. Delete duplicated lines 113-127 from `run_analysis`
  4. Run `make quality` to verify

  **Must NOT do**:
  - Change `run_analysis_returning_report` implementation
  - Change any public function signatures
  - Modify output formatting logic

  **Parallelizable**: NO (depends on 0)

  **References**:
  - `src/lib.rs:110-130` - current `run_analysis` implementation
  - `src/lib.rs:136-156` - `run_analysis_returning_report` (the one to call)
  - `src/lib.rs:472-484` - `test_run_analysis_with_verbose_and_max_lines` (characterization test)

  **Acceptance Criteria**:
  - [ ] `run_analysis` body is exactly 2 lines (doc + call)
  - [ ] `make quality` → PASS
  - [ ] Existing test `test_run_analysis_with_verbose_and_max_lines` passes

  **Commit**: YES
  - Message: `refactor(lib): deduplicate run_analysis entry points`
  - Files: `src/lib.rs`
  - Pre-commit: `make quality`

---

- [x] 2. [cq-002] Language Configuration Strategy Pattern

  **What to do**:
  1. Add characterization test: call each `NodeKindMapper` method for all 9 languages, assert non-empty results
  2. Define `LanguageSpec` struct with all node kind fields as `&'static [&'static str]`
  3. Create static constants: `RUST_SPEC`, `JS_SPEC`, etc. for each of 9 languages
  4. Add `fn spec(&self) -> &'static LanguageSpec` to `SupportedLanguage`
  5. Refactor `NodeKindMapper` methods to delegate to `self.spec().field`
  6. Verify all existing tests pass

  **Must NOT do**:
  - Change `NodeKindMapper` trait signature
  - Change node kind values (copy exactly from match arms)
  - Add new languages
  - Change `LanguageManager` logic

  **Parallelizable**: NO (depends on 1)

  **References**:
  - `src/analyzer/language.rs:142-411` - current `NodeKindMapper` impl with match blocks
  - `src/analyzer/language.rs:155-177` - `function_node_kinds` example pattern
  - `src/analyzer/language.rs:597-607` - `test_node_kind_mapping` (characterization)

  **Acceptance Criteria**:
  - [ ] `LanguageSpec` struct exists with 9 fields
  - [ ] 9 static constants defined (one per language)
  - [ ] `spec()` method returns correct spec for each variant
  - [ ] All match blocks in `NodeKindMapper` reduced to single delegation
  - [ ] `make quality` → PASS
  - [ ] `test_node_kind_mapping` passes

  **Commit**: YES
  - Message: `refactor(language): introduce LanguageSpec strategy pattern`
  - Files: `src/analyzer/language.rs`
  - Pre-commit: `make quality`

---

- [x] 3. [cq-001] Extract Sanitizer Module

  **What to do**:
  1. Create `src/analyzer/sanitizer.rs` with:
     - `pub(crate) fn sanitize_for_tree_sitter(source: &str, language: SupportedLanguage) -> Cow<'_, str>`
     - `fn escape_ampersands_in_jsx_text(source: &str) -> String` (private)
  2. Move functions from `parser.rs:543-807` to new module
  3. Add `pub mod sanitizer;` to `src/analyzer/mod.rs`
  4. Update `parser.rs` to import: `use crate::analyzer::sanitizer::sanitize_for_tree_sitter;`
  5. Move sanitizer tests from `parser.rs:1248-1290` to `sanitizer.rs`
  6. Verify all tests pass

  **Must NOT do**:
  - Change sanitization logic
  - Change function signatures
  - Move other code from parser.rs
  - Change parser.rs public API

  **Parallelizable**: NO (depends on 2)

  **References**:
  - `src/analyzer/parser.rs:543-573` - `sanitize_for_tree_sitter` to move
  - `src/analyzer/parser.rs:575-807` - `escape_ampersands_in_jsx_text` to move
  - `src/analyzer/parser.rs:484` - usage site (must update import)
  - `src/analyzer/parser.rs:1248-1290` - sanitizer tests to move

  **Acceptance Criteria**:
  - [ ] `src/analyzer/sanitizer.rs` exists with ~270 lines
  - [ ] `sanitize_for_tree_sitter` is `pub(crate)`
  - [ ] `escape_ampersands_in_jsx_text` is private to module
  - [ ] `parser.rs` line count reduced by ~270
  - [ ] `make quality` → PASS
  - [ ] Sanitizer tests pass in new location

  **Commit**: YES
  - Message: `refactor(analyzer): extract sanitizer module from parser`
  - Files: `src/analyzer/sanitizer.rs`, `src/analyzer/parser.rs`, `src/analyzer/mod.rs`
  - Pre-commit: `make quality`

---

- [x] 4. [cq-005] Parser Reuse in Parallel Analysis

  **What to do**:
  1. Add characterization test: analyze a project with multiple files, verify results match
  2. Replace `Arc<Mutex<Vec>>` pattern with `map_init` in `analyze_files_parallel`
  3. Use `map_init(|| FileParser::new(...), |parser, file| ...)` for thread-local reuse
  4. Collect results with `.collect::<Vec<_>>()` and partition Ok/Err after
  5. Verify identical output on test fixture

  **Must NOT do**:
  - Change `FileParser` API
  - Change result format or error handling semantics
  - Remove progress bar functionality
  - Change any public analyzer API

  **Parallelizable**: NO (depends on 3, final task)

  **References**:
  - `src/analyzer/mod.rs:142-264` - `analyze_files_parallel` to refactor
  - `src/analyzer/mod.rs:161-163` - current mutex pattern
  - `src/analyzer/mod.rs:170-229` - parallel loop to refactor
  - `src/analyzer/mod.rs:489-507` - `test_analyze_project` (characterization)
  - rayon docs: `par_iter().map_init(init, f)` pattern

  **Acceptance Criteria**:
  - [ ] No `Arc<Mutex<Vec>>` for results (warnings/errors may still use mutex for logging)
  - [ ] Uses `map_init` with `FileParser::new` as init closure
  - [ ] `make quality` → PASS
  - [ ] `test_analyze_project` produces identical results
  - [ ] `test_cli_filters` passes

  **Commit**: YES
  - Message: `perf(analyzer): reuse parsers per-thread via map_init`
  - Files: `src/analyzer/mod.rs`
  - Pre-commit: `make quality`

---

## Commit Strategy

| After Task | Message | Files | Verification |
|------------|---------|-------|--------------|
| 1 | `refactor(lib): deduplicate run_analysis entry points` | lib.rs | make quality |
| 2 | `refactor(language): introduce LanguageSpec strategy pattern` | language.rs | make quality |
| 3 | `refactor(analyzer): extract sanitizer module from parser` | sanitizer.rs, parser.rs, mod.rs | make quality |
| 4 | `perf(analyzer): reuse parsers per-thread via map_init` | mod.rs | make quality |

---

## Success Criteria

### Verification Commands
```bash
make quality                    # Expected: 0 errors, 0 warnings
cargo test                      # Expected: all tests pass
cargo test -- --nocapture 2>&1 | grep "test result"  # Expected: same count as baseline
```

### Final Checklist
- [ ] All 4 refactors complete with commits
- [ ] `make quality` passes
- [ ] Test count unchanged from baseline
- [ ] No public API changes
- [ ] No JSON schema changes
