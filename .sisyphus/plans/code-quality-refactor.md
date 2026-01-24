# Code Quality Refactoring

## Context

### Original Request
Implement code quality improvements from analysis of `shotgun-prompt-20260124-154859_response.md`:
1. DRY fix in lib.rs (eliminate duplicate execution logic)
2. Improve panic messages in walker.rs
3. (Deferred) JSX sanitizer extraction - requires tests first
4. (Deferred) Parser decomposition - requires JSX extraction first

### Interview Summary
**Key Discussions**:
- Phase 1 focuses on quick wins with low risk
- Phases 2-3 deferred pending test coverage
- cq-003 (trait-based language registry) permanently deferred as over-engineered

**Research Findings**:
- `run_analysis` and `run_analysis_returning_report` share ~90% code (lib.rs:82-182)
- walker.rs has multiple `.unwrap()` calls that should use `.expect()`
- mod.rs also has unwrap patterns needing improvement

---

## Work Objectives

### Core Objective
Improve code quality through DRY refactoring and better error messages.

### Concrete Deliverables
- Single `execute_analysis_core()` helper extracted in lib.rs
- All `.unwrap()` calls in walker.rs and mod.rs replaced with `.expect()` with context

### Definition of Done
- [x] lib.rs has no duplicate analysis logic
- [x] walker.rs uses `.expect()` with descriptive messages
- [x] mod.rs uses `.expect()` with descriptive messages
- [x] `make quality` passes
- [x] All existing tests pass

### Must Have
- No behavior changes
- Better panic messages for debugging

### Must NOT Have (Guardrails)
- New dependencies
- Changes to public API signatures
- Error propagation for mutex poison (not recoverable)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (cargo test, make quality)
- **User wants tests**: Existing tests sufficient
- **Framework**: Rust standard

---

## Task Flow

```
Task 1 (DRY lib.rs) → Task 2 (walker.rs expects) → Task 3 (mod.rs expects) → Task 4 (Verification)
```

## Parallelization

| Task | Depends On | Reason |
|------|------------|--------|
| 1 | None | Independent refactor |
| 2 | None | Independent refactor |
| 3 | None | Independent refactor |
| 4 | 1, 2, 3 | Verification after all changes |

**Tasks 1, 2, 3 are parallelizable.**

---

## TODOs

- [x] 1. Extract Common Analysis Logic in lib.rs (cq-002)

  **What to do**:
  - Create private `execute_analysis_core(args: &CliArgs) -> Result<AnalysisReport>` function
  - Move shared logic from `run_analysis` and `run_analysis_returning_report`
  - Keep output handling in the public functions (they differ)
  - Update both public functions to call the helper

  **Must NOT do**:
  - Change public API signatures
  - Modify output handling logic

  **Parallelizable**: YES (with 2, 3)

  **References**:
  - `src/lib.rs:82-129` - run_analysis function
  - `src/lib.rs:135-182` - run_analysis_returning_report function

  **Acceptance Criteria**:
  - [x] `execute_analysis_core` exists as private function
  - [x] Both public functions call it
  - [x] No duplicate validation/setup code
  - [x] `cargo test` passes

  **Implementation Details**:
  ```rust
  // New private helper (add after imports, before run_analysis):
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

  // Simplified run_analysis:
  pub fn run_analysis(args: CliArgs) -> Result<()> {
      let report = execute_analysis_core(&args)?;

      if args.compact {
          output::display_compact_table(&report.files, args.sort, args.limit);
      } else {
          let output_manager = OutputManager::from_cli_args(&args);
          output_manager.generate_output(&report, &args)?;
      }

      if args.verbose {
          println!();
          println!("Analysis completed successfully!");
          println!("Files analyzed: {}", report.files.len());
          println!("Total lines: {}", report.summary.total_lines);
          println!("Total functions: {}", report.summary.total_functions);
          println!("Total classes: {}", report.summary.total_classes);
      }

      Ok(())
  }

  // Simplified run_analysis_returning_report:
  pub fn run_analysis_returning_report(args: CliArgs) -> Result<AnalysisReport> {
      let report = execute_analysis_core(&args)?;

      if args.compact {
          output::display_compact_table(&report.files, args.sort, args.limit);
      } else {
          let output_manager = OutputManager::from_cli_args(&args);
          output_manager.generate_output(&report, &args)?;
      }

      if args.verbose {
          println!();
          println!("Analysis completed successfully!");
          println!("Files analyzed: {}", report.files.len());
          println!("Total lines: {}", report.summary.total_lines);
          println!("Total functions: {}", report.summary.total_functions);
          println!("Total classes: {}", report.summary.total_classes);
      }

      Ok(report)
  }
  ```

  **Commit**: YES
  - Message: `refactor(lib): extract common analysis logic to eliminate duplication`
  - Files: `src/lib.rs`
  - Pre-commit: `make quality`

---

- [x] 2. Replace .unwrap() with .expect() in walker.rs (cq-005)

  **What to do**:
  - Find all `.unwrap()` calls on Mutex locks
  - Replace with `.expect("descriptive message")`
  - Find all `Arc::try_unwrap().unwrap()` patterns
  - Add descriptive expect messages

  **Must NOT do**:
  - Add error propagation (mutex poison is not recoverable)
  - Change control flow

  **Parallelizable**: YES (with 1, 3)

  **References**:
  - `src/analyzer/walker.rs:148` - stats.lock().unwrap()
  - `src/analyzer/walker.rs:172` - files.lock().unwrap()
  - `src/analyzer/walker.rs:185` - errors.lock().unwrap()
  - `src/analyzer/walker.rs:192` - errors.lock().unwrap()
  - `src/analyzer/walker.rs:205-209` - Arc::try_unwrap chains

  **Acceptance Criteria**:
  - [x] No `.unwrap()` on Mutex locks in walker.rs
  - [x] All `.expect()` have descriptive messages
  - [x] `cargo test` passes

  **Implementation Details**:
  ```rust
  // Line 148: stats.lock().unwrap() -> 
  stats.lock().expect("walk stats mutex poisoned")

  // Line 172: files.lock().unwrap() ->
  files.lock().expect("discovered files mutex poisoned")

  // Line 185: errors.lock().unwrap() ->
  errors.lock().expect("walk errors mutex poisoned")

  // Line 192: errors.lock().unwrap() ->
  errors.lock().expect("walk errors mutex poisoned")

  // Lines 205-209: Arc chain
  let files = Arc::try_unwrap(files)
      .expect("files Arc still has multiple owners after walk completed")
      .into_inner()
      .expect("files mutex poisoned");

  let stats = Arc::try_unwrap(stats)
      .expect("stats Arc still has multiple owners after walk completed")
      .into_inner()
      .expect("stats mutex poisoned");

  let errors = Arc::try_unwrap(errors)
      .expect("errors Arc still has multiple owners after walk completed")
      .into_inner()
      .expect("errors mutex poisoned");
  ```

  **Commit**: YES
  - Message: `refactor(walker): improve panic messages with descriptive .expect()`
  - Files: `src/analyzer/walker.rs`
  - Pre-commit: `make quality`

---

- [x] 3. Replace .unwrap() with .expect() in mod.rs (cq-005 continued)

  **What to do**:
  - Find all `.unwrap()` calls on Mutex/Arc in mod.rs
  - Replace with `.expect("descriptive message")`

  **Must NOT do**:
  - Add error propagation
  - Change control flow

  **Parallelizable**: YES (with 1, 2)

  **References**:
  - `src/analyzer/mod.rs` - Search for `.unwrap()` patterns

  **Acceptance Criteria**:
  - [x] No `.unwrap()` on Mutex/Arc in mod.rs
  - [x] All `.expect()` have descriptive messages
  - [x] `cargo test` passes

  **Commit**: YES
  - Message: `refactor(analyzer): improve panic messages with descriptive .expect()`
  - Files: `src/analyzer/mod.rs`
  - Pre-commit: `make quality`

---

- [x] 4. Final Verification

  **What to do**:
  - Run `make quality` (fmt + clippy + test)
  - Verify no behavior changes
  - Check all .unwrap() on Mutex/Arc are replaced

  **Acceptance Criteria**:
  - [x] `make quality` passes
  - [x] `cargo test` passes
  - [x] No clippy warnings

  **Commit**: NO (verification only)

---

## Commit Strategy

| After Task | Message | Files |
|------------|---------|-------|
| 1 | `refactor(lib): extract common analysis logic to eliminate duplication` | lib.rs |
| 2 | `refactor(walker): improve panic messages with descriptive .expect()` | walker.rs |
| 3 | `refactor(analyzer): improve panic messages with descriptive .expect()` | mod.rs |

---

## Success Criteria

### Verification Commands
```bash
make quality                           # All checks pass
cargo test                             # All tests pass
grep -n "\.unwrap()" src/analyzer/walker.rs | grep -v "test"  # Should be minimal/none on Mutex
```

### Final Checklist
- [x] DRY fix applied in lib.rs
- [x] All .expect() have descriptive messages
- [x] All tests pass
- [x] No clippy warnings
