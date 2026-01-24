# Code Analyzer Improvements

## Context

### Original Request
Implement 4 features from `shotgun-prompt-20260124-151502_response.md`:
1. Single file analysis support
2. Sort by Methods capability
3. Nesting Depth metric
4. CSV output format

### Interview Summary
**Key Discussions**:
- Features are ordered by dependency and effort
- Phase 1 (Quick Wins): Single file + Sort by Methods
- Phase 2 (Core): Nesting Depth + CSV Output

**Research Findings**:
- `discover_files` in walker.rs currently rejects files at line 109
- `SortBy` enum in cli.rs has 6 variants (line 181-195)
- `apply_sorting` in terminal.rs (line 470) implements sorting
- `FileAnalysis` struct needs `max_nesting_depth` field
- CSV crate may need to be added to Cargo.toml

---

## Work Objectives

### Core Objective
Extend code-analyzer with 4 new features: single file analysis, methods sorting, nesting depth metric, and CSV export.

### Concrete Deliverables
- Single file paths accepted as input
- `--sort methods` CLI option
- `max_nesting_depth` field in analysis output
- `--output csv` format option

### Definition of Done
- [x] `code-analyzer src/main.rs` works (single file)
- [x] `code-analyzer --sort methods` sorts by method count
- [x] JSON output includes `max_nesting_depth` field
- [x] `code-analyzer --output csv` produces valid CSV
- [x] `make quality` passes

### Must Have
- All existing tests pass
- New features have tests
- CLI help updated

### Must NOT Have (Guardrails)
- Breaking changes to existing JSON schema
- New dependencies without justification
- Recursive AST traversal (use TreeCursor)

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES (cargo test)
- **User wants tests**: YES (TDD-like)
- **Framework**: Rust standard + tempfile

---

## Task Flow

```
Task 1 (Single File) → Task 2 (Sort Methods) → Task 3 (Nesting Depth) → Task 4 (CSV)
```

## Parallelization

| Task | Depends On | Reason |
|------|------------|--------|
| 1 | None | Foundation |
| 2 | None | Independent |
| 3 | None | Independent |
| 4 | 3 | CSV needs nesting_depth field |

---

## TODOs

- [x] 1. Implement Single File Analysis Support

  **What to do**:
  - Modify `CliArgs::validate()` in `src/cli.rs` to allow files (not just directories)
  - Modify `FileWalker::discover_files()` in `src/analyzer/walker.rs` to handle single files
  - Add `discover_single_file()` helper method
  - Return clear error for unsupported file extensions

  **Must NOT do**:
  - Change directory traversal logic
  - Modify language detection

  **Parallelizable**: YES (with 2, 3)

  **References**:
  - `src/cli.rs:241-253` - Current validation that rejects files
  - `src/cli.rs:26-27` - PATH argument definition
  - `src/analyzer/walker.rs:99-114` - Current discover_files that rejects files
  - `src/analyzer/walker.rs:269-306` - `should_include_file` for validation logic
  - `src/analyzer/language.rs` - `LanguageManager::is_supported_file()`

  **Acceptance Criteria**:
  - [ ] `cargo run -- src/main.rs` analyzes single file
  - [ ] `cargo run -- src/nonexistent.rs` returns clear error
  - [ ] `cargo run -- README.md` returns "unsupported file type" error
  - [ ] `cargo test` passes
  
  **Implementation Details**:
  ```rust
  // In cli.rs validate():
  // Remove the is_dir() check, allow files
  if let Some(ref path) = self.path {
      if !path.exists() {
          return Err(AnalyzerError::invalid_path(path));
      }
      // Files and directories both allowed now
  }
  
  // In walker.rs discover_files():
  if root_path.is_file() {
      return self.discover_single_file(root_path);
  }
  
  // New method:
  fn discover_single_file(&self, file_path: &Path) -> Result<(Vec<PathBuf>, WalkStats)> {
      if !self.language_manager.is_supported_file(file_path) {
          return Err(AnalyzerError::validation_error(format!(
              "Unsupported file type: {}", file_path.display()
          )));
      }
      // Check file size
      // Return vec![file_path] with stats
  }
  ```

  **Commit**: YES
  - Message: `feat(cli): support single file analysis`
  - Files: `src/cli.rs`, `src/analyzer/walker.rs`
  - Pre-commit: `make quality`

---

- [x] 2. Add Sort by Methods Capability

  **What to do**:
  - Add `Methods` variant to `SortBy` enum in `src/cli.rs`
  - Add match arm in `apply_sorting()` in `src/output/terminal.rs`
  - Update `Display` impl for `SortBy`

  **Must NOT do**:
  - Modify FileAnalysis struct (methods field already exists)

  **Parallelizable**: YES (with 1, 3)

  **References**:
  - `src/cli.rs:181-195` - SortBy enum definition
  - `src/cli.rs:197-208` - SortBy Display implementation
  - `src/output/terminal.rs:469-500` - apply_sorting function
  - `src/analyzer/parser.rs` - FileAnalysis has `methods: usize` field

  **Acceptance Criteria**:
  - [ ] `code-analyzer --sort methods` works
  - [ ] `code-analyzer --help` shows "methods" option
  - [ ] Results sorted by method count descending
  - [ ] `cargo test` passes

  **Implementation Details**:
  ```rust
  // In cli.rs SortBy enum:
  /// Sort by number of methods (descending)
  Methods,
  
  // In cli.rs Display impl:
  SortBy::Methods => write!(f, "methods"),
  
  // In terminal.rs apply_sorting():
  SortBy::Methods => {
      files.sort_by_key(|f| std::cmp::Reverse(f.methods));
  }
  ```

  **Commit**: YES
  - Message: `feat(cli): add --sort methods option`
  - Files: `src/cli.rs`, `src/output/terminal.rs`
  - Pre-commit: `make quality`

---

- [x] 3. Add Nesting Depth Metric

  **What to do**:
  - Add `nesting_node_kinds()` method to `NodeKindMapper` trait in `src/analyzer/language.rs`
  - Implement for each `SupportedLanguage`
  - Create `calculate_max_nesting_depth()` in `src/analyzer/parser.rs`
  - Add `max_nesting_depth: usize` to `FileAnalysis` struct
  - Wire up in `parse_file_metrics()`
  - Update JSON serialization (automatic via serde)

  **Must NOT do**:
  - Count function definitions as nesting (they reset context)
  - Use recursion (use TreeCursor)

  **Parallelizable**: YES (with 1, 2)

  **References**:
  - `src/analyzer/language.rs:50-100` - NodeKindMapper trait
  - `src/analyzer/language.rs:195+` - control_flow_node_kinds() for pattern
  - `src/analyzer/parser.rs:853` - count_nodes_iterative for TreeCursor pattern
  - `src/analyzer/parser.rs:100-120` - FileAnalysis struct definition

  **Acceptance Criteria**:
  - [ ] `max_nesting_depth` field in JSON output
  - [ ] Deeply nested files show higher values
  - [ ] Value is 0 for flat files
  - [ ] `cargo test` passes

  **Implementation Details**:
  ```rust
  // In language.rs trait:
  fn nesting_node_kinds(&self) -> &'static [&'static str];
  
  // Rust implementation:
  fn nesting_node_kinds(&self) -> &'static [&'static str] {
      &["if_expression", "match_expression", "for_expression", 
        "while_expression", "loop_expression", "block"]
  }
  
  // In parser.rs:
  fn calculate_max_nesting_depth(root: &Node, mapper: &dyn NodeKindMapper) -> usize {
      let nesting_kinds = mapper.nesting_node_kinds();
      let mut max_depth = 0;
      let mut current_depth = 0;
      let mut cursor = root.walk();
      
      loop {
          let node = cursor.node();
          if nesting_kinds.contains(&node.kind()) {
              current_depth += 1;
              max_depth = max_depth.max(current_depth);
          }
          
          if cursor.goto_first_child() { continue; }
          loop {
              // Decrement depth when leaving nesting node
              if nesting_kinds.contains(&cursor.node().kind()) {
                  current_depth = current_depth.saturating_sub(1);
              }
              if cursor.goto_next_sibling() { break; }
              if !cursor.goto_parent() { return max_depth; }
              if nesting_kinds.contains(&cursor.node().kind()) {
                  current_depth = current_depth.saturating_sub(1);
              }
          }
      }
  }
  
  // In FileAnalysis:
  pub max_nesting_depth: usize,
  ```

  **Commit**: YES
  - Message: `feat(metrics): add max nesting depth calculation`
  - Files: `src/analyzer/language.rs`, `src/analyzer/parser.rs`
  - Pre-commit: `make quality`

---

- [x] 4. Implement CSV Output Format

  **What to do**:
  - Add `csv` crate to `Cargo.toml`
  - Add `Csv` variant to `OutputFormat` enum in `src/cli.rs`
  - Create `src/output/csv.rs` with `CsvExporter`
  - Add `mod csv;` to `src/output/mod.rs`
  - Update `OutputManager::generate_output()` for CSV

  **Must NOT do**:
  - Change existing output formats
  - Make CSV the default

  **Parallelizable**: NO (depends on 3 for complete schema)

  **References**:
  - `src/cli.rs:210-225` - OutputFormat enum
  - `src/output/mod.rs:1-12` - Module exports
  - `src/output/mod.rs:52-105` - generate_output function
  - `src/output/json.rs` - Pattern for exporter struct

  **Acceptance Criteria**:
  - [ ] `code-analyzer --output csv` produces valid CSV
  - [ ] Headers match FileAnalysis fields
  - [ ] `--output-file report.csv` saves to file
  - [ ] `cargo test` passes

  **Implementation Details**:
  ```toml
  # Cargo.toml
  csv = "1.3"
  ```
  
  ```rust
  // In cli.rs OutputFormat:
  /// CSV output
  Csv,
  
  // Display impl:
  OutputFormat::Csv => write!(f, "csv"),
  
  // src/output/csv.rs:
  use csv::Writer;
  use std::path::Path;
  use crate::analyzer::FileAnalysis;
  use crate::error::Result;
  
  pub struct CsvExporter;
  
  impl CsvExporter {
      pub fn export_to_file(files: &[FileAnalysis], path: &Path) -> Result<()> {
          let mut wtr = Writer::from_path(path)?;
          wtr.write_record(&[
              "path", "language", "lines_of_code", "blank_lines", 
              "comment_lines", "functions", "methods", "classes",
              "cyclomatic_complexity", "complexity_score", "max_nesting_depth"
          ])?;
          for f in files {
              wtr.write_record(&[
                  f.path.display().to_string(),
                  f.language.clone(),
                  f.lines_of_code.to_string(),
                  // ... other fields
              ])?;
          }
          wtr.flush()?;
          Ok(())
      }
  }
  
  // In output/mod.rs generate_output():
  OutputFormat::Csv => {
      let path = args.csv_output_path(); // or reuse json_output_path with .csv
      csv::CsvExporter::export_to_file(&report.files, &path)?;
  }
  ```

  **Commit**: YES
  - Message: `feat(output): add CSV export format`
  - Files: `Cargo.toml`, `src/cli.rs`, `src/output/mod.rs`, `src/output/csv.rs`
  - Pre-commit: `make quality`

---

- [x] 5. Final Verification

  **What to do**:
  - Run `make quality` (fmt + clippy + test)
  - Manual testing of all 4 features
  - Verify JSON backward compatibility

  **Acceptance Criteria**:
  - [ ] `make quality` passes
  - [ ] `code-analyzer src/main.rs` works
  - [ ] `code-analyzer --sort methods` works
  - [ ] JSON output has `max_nesting_depth`
  - [ ] `code-analyzer --output csv` works

  **Commit**: NO (verification only)

---

## Commit Strategy

| After Task | Message | Files |
|------------|---------|-------|
| 1 | `feat(cli): support single file analysis` | cli.rs, walker.rs |
| 2 | `feat(cli): add --sort methods option` | cli.rs, terminal.rs |
| 3 | `feat(metrics): add max nesting depth calculation` | language.rs, parser.rs |
| 4 | `feat(output): add CSV export format` | Cargo.toml, cli.rs, output/*.rs |

---

## Success Criteria

### Verification Commands
```bash
make quality                           # All checks pass
code-analyzer src/main.rs              # Single file works
code-analyzer --sort methods .         # Methods sorting works
code-analyzer --output json . | jq '.files[0].max_nesting_depth'  # Field exists
code-analyzer --output csv .           # CSV output works
```

### Final Checklist
- [x] All 4 features implemented
- [x] All tests pass
- [x] No clippy warnings
- [x] JSON schema backward compatible
