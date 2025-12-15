# Code Analyzer v3.0 - Implementation Plan

## Executive Summary

This comprehensive plan details the implementation of new features for Code Analyzer, prioritized by business value and technical feasibility. The analysis of `new-features.md` and the current codebase reveals that several foundational features have already been implemented in v2.0. This plan focuses on the **remaining high-value features** that will differentiate the tool and enable enterprise/CI/CD adoption.

---

## Current State Analysis

### Already Implemented (v2.0)
| Feature | Location | Status |
|---------|----------|--------|
| Cyclomatic Complexity (AST-based) | `src/analyzer/parser.rs:488-497` | Done |
| AST Comment Line Counting | `src/analyzer/parser.rs:501-539` | Done |
| Iterative AST Traversal | `src/analyzer/parser.rs:435-464` | Done |
| Structured ParseWarning | `src/error.rs:6-70` | Done |
| Methods vs Functions separation | `src/analyzer/language.rs:280-297` | Done |
| Compact CLI mode | `src/cli.rs:93-98` | Done |
| Color mode control | `src/cli.rs:112-119` | Done |
| Relative path display | `src/output/terminal.rs:412-431` | Done |
| Severity indicators | `src/output/terminal.rs:52-61` | Done |
| Refactoring candidates display | `src/output/terminal.rs:308-365` | Done |
| Metrics legend | `src/output/terminal.rs:64-70` | Done |
| Walk stats tracking | `src/analyzer/walker.rs:45-64` | Done |

### Remaining Features to Implement
| Feature | Priority | Complexity | Business Value |
|---------|----------|------------|----------------|
| Custom Refactoring Thresholds | Critical | S | Enables enterprise adoption |
| Git Integration (--only-changed) | High | M | CI/CD performance 10x+ |
| Enhanced JSON Output Options | Medium | S | API/dashboard integration |
| CI Mode with Exit Codes | Medium | S | Pipeline integration |
| Walk Stats in Summary Display | Low | S | Transparency improvement |
| Windows Path Normalization | Low | S | Cross-platform consistency |

---

## Phase 1: Custom Refactoring Thresholds (Critical Priority)

### Problem Statement
The current refactoring candidate thresholds are hardcoded in `src/analyzer/parser.rs:147-184`:
```rust
complexity_score >= 10.0     // High priority
cyclomatic_complexity >= 20  // High CC
lines_of_code >= 500         // Large file
functions >= 20              // Too many functions
```

Different organizations have different coding standards. A startup may tolerate higher complexity, while a financial institution requires stricter limits. Without configurability, the tool cannot adapt to organizational needs.

### Solution
Add CLI flags to override each threshold, with sensible defaults maintaining backward compatibility.

### Implementation Steps

#### Task 1.1: Add CLI Arguments
**File:** `src/cli.rs`
**Location:** After line 119 (after `color` field)

```rust
/// Maximum complexity score threshold for refactoring candidates
#[arg(
    long,
    value_name = "SCORE",
    help = "Complexity score threshold for refactoring candidates (default: 10.0)"
)]
pub max_complexity_score: Option<f64>,

/// Maximum cyclomatic complexity threshold
#[arg(
    long,
    value_name = "CC",
    help = "Cyclomatic complexity threshold for refactoring candidates (default: 20)"
)]
pub max_cc: Option<usize>,

/// Maximum lines of code threshold
#[arg(
    long,
    value_name = "LINES",
    help = "Lines of code threshold for large file detection (default: 500)"
)]
pub max_loc: Option<usize>,

/// Maximum functions per file threshold
#[arg(
    long,
    value_name = "COUNT",
    help = "Function count threshold for refactoring candidates (default: 20)"
)]
pub max_functions_per_file: Option<usize>,
```

**Location:** Update `Default` impl (around line 478)
```rust
max_complexity_score: None,
max_cc: None,
max_loc: None,
max_functions_per_file: None,
```

#### Task 1.2: Create RefactoringThresholds Struct
**File:** `src/analyzer/parser.rs`
**Location:** After line 144 (before `identify_refactoring_candidates`)

```rust
/// Configurable thresholds for identifying refactoring candidates
#[derive(Debug, Clone)]
pub struct RefactoringThresholds {
    /// Complexity score threshold (default: 10.0)
    pub max_complexity_score: f64,
    /// Cyclomatic complexity threshold (default: 20)
    pub max_cyclomatic_complexity: usize,
    /// Lines of code threshold (default: 500)
    pub max_lines_of_code: usize,
    /// Functions per file threshold (default: 20)
    pub max_functions: usize,
}

impl Default for RefactoringThresholds {
    fn default() -> Self {
        Self {
            max_complexity_score: 10.0,
            max_cyclomatic_complexity: 20,
            max_lines_of_code: 500,
            max_functions: 20,
        }
    }
}

impl RefactoringThresholds {
    /// Create thresholds from CLI arguments, using defaults for unspecified values
    pub fn from_cli(args: &crate::cli::CliArgs) -> Self {
        Self {
            max_complexity_score: args.max_complexity_score.unwrap_or(10.0),
            max_cyclomatic_complexity: args.max_cc.unwrap_or(20),
            max_lines_of_code: args.max_loc.unwrap_or(500),
            max_functions: args.max_functions_per_file.unwrap_or(20),
        }
    }
}
```

#### Task 1.3: Update identify_refactoring_candidates Signature
**File:** `src/analyzer/parser.rs`
**Location:** Line 147

Change from:
```rust
pub fn identify_refactoring_candidates(files: &[FileAnalysis]) -> Vec<RefactoringCandidate> {
```

To:
```rust
/// Identify files that are candidates for refactoring based on configurable thresholds
pub fn identify_refactoring_candidates(
    files: &[FileAnalysis],
    thresholds: &RefactoringThresholds,
) -> Vec<RefactoringCandidate> {
    let mut candidates = Vec::new();

    for file in files {
        let mut reasons = Vec::new();

        // Check complexity score against threshold
        if file.complexity_score >= thresholds.max_complexity_score {
            reasons.push(RefactoringReason::HighComplexityScore(file.complexity_score));
        }

        // Check cyclomatic complexity against threshold
        if file.cyclomatic_complexity >= thresholds.max_cyclomatic_complexity {
            reasons.push(RefactoringReason::HighCyclomaticComplexity(file.cyclomatic_complexity));
        }

        // Check file size against threshold
        if file.lines_of_code >= thresholds.max_lines_of_code {
            reasons.push(RefactoringReason::LargeFile(file.lines_of_code));
        }

        // Check function count against threshold
        if file.functions >= thresholds.max_functions {
            reasons.push(RefactoringReason::TooManyFunctions(file.functions));
        }

        if !reasons.is_empty() {
            candidates.push(RefactoringCandidate {
                file: file.clone(),
                reasons,
            });
        }
    }

    // Sort by complexity score (highest first)
    candidates.sort_by(|a, b| {
        b.file.complexity_score
            .partial_cmp(&a.file.complexity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates
}
```

#### Task 1.4: Update All Call Sites
**Files to update:**

1. **`src/output/terminal.rs:89`**
   ```rust
   // Before
   let candidates = identify_refactoring_candidates(&report.files);

   // After - need to pass thresholds from somewhere
   let thresholds = RefactoringThresholds::default(); // Temporary
   let candidates = identify_refactoring_candidates(&report.files, &thresholds);
   ```

2. **`src/analyzer/mod.rs`** - Add thresholds to AnalyzerEngine or pass through
   ```rust
   pub struct AnalyzerEngine {
       // ... existing fields
       refactoring_thresholds: RefactoringThresholds,
   }
   ```

3. **`src/output/mod.rs`** - Store thresholds in OutputManager
   ```rust
   pub struct OutputManager {
       // ... existing fields
       thresholds: RefactoringThresholds,
   }
   ```

4. **`src/lib.rs`** - Export RefactoringThresholds
   ```rust
   pub use analyzer::parser::RefactoringThresholds;
   ```

#### Task 1.5: Add Tests
**File:** `src/analyzer/parser.rs` (in tests module)

```rust
#[test]
fn test_refactoring_thresholds_default() {
    let thresholds = RefactoringThresholds::default();
    assert_eq!(thresholds.max_complexity_score, 10.0);
    assert_eq!(thresholds.max_cyclomatic_complexity, 20);
    assert_eq!(thresholds.max_lines_of_code, 500);
    assert_eq!(thresholds.max_functions, 20);
}

#[test]
fn test_refactoring_thresholds_from_cli() {
    let args = crate::cli::CliArgs {
        max_complexity_score: Some(5.0),
        max_cc: Some(10),
        max_loc: Some(200),
        max_functions_per_file: Some(8),
        ..Default::default()
    };

    let thresholds = RefactoringThresholds::from_cli(&args);
    assert_eq!(thresholds.max_complexity_score, 5.0);
    assert_eq!(thresholds.max_cyclomatic_complexity, 10);
    assert_eq!(thresholds.max_lines_of_code, 200);
    assert_eq!(thresholds.max_functions, 8);
}

#[test]
fn test_identify_refactoring_candidates_with_custom_thresholds() {
    let files = vec![
        FileAnalysis {
            path: PathBuf::from("moderate.rs"),
            language: "rust".to_string(),
            lines_of_code: 300,
            blank_lines: 30,
            comment_lines: 20,
            functions: 12,
            methods: 5,
            classes: 2,
            cyclomatic_complexity: 15,
            complexity_score: 8.0,
        },
    ];

    // With default thresholds - should NOT be a candidate
    let default_thresholds = RefactoringThresholds::default();
    let candidates = identify_refactoring_candidates(&files, &default_thresholds);
    assert!(candidates.is_empty(), "File should not be candidate with default thresholds");

    // With strict thresholds - SHOULD be a candidate
    let strict_thresholds = RefactoringThresholds {
        max_complexity_score: 5.0,
        max_cyclomatic_complexity: 10,
        max_lines_of_code: 200,
        max_functions: 10,
    };
    let candidates = identify_refactoring_candidates(&files, &strict_thresholds);
    assert_eq!(candidates.len(), 1, "File should be candidate with strict thresholds");
    assert!(candidates[0].reasons.len() >= 3, "Should have multiple reasons");
}

#[test]
fn test_identify_refactoring_candidates_partial_thresholds() {
    let files = vec![
        FileAnalysis {
            path: PathBuf::from("large_but_simple.rs"),
            language: "rust".to_string(),
            lines_of_code: 800,  // Exceeds 500
            blank_lines: 50,
            comment_lines: 100,
            functions: 5,        // Low
            methods: 2,
            classes: 1,
            cyclomatic_complexity: 8,  // Low
            complexity_score: 4.0,     // Low
        },
    ];

    let thresholds = RefactoringThresholds::default();
    let candidates = identify_refactoring_candidates(&files, &thresholds);

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].reasons.len(), 1);
    assert!(matches!(
        candidates[0].reasons[0],
        RefactoringReason::LargeFile(800)
    ));
}
```

### Acceptance Criteria
- [ ] `code-analyzer --max-cc 10` flags files with CC >= 10
- [ ] `code-analyzer --max-loc 200` flags files with >= 200 lines
- [ ] `code-analyzer --max-complexity-score 5.0` flags files with score >= 5.0
- [ ] `code-analyzer --max-functions-per-file 10` flags files with >= 10 functions
- [ ] Omitted flags use hardcoded defaults (backward compatible)
- [ ] All existing tests pass
- [ ] New tests achieve >80% coverage on new code

---

## Phase 2: Git Integration for Analysis Delta (High Priority)

### Problem Statement
Currently, the analyzer processes all files in a directory on every run. In CI/CD pipelines, this is wasteful - typically only a small subset of files changes between commits. For large codebases (10K+ files), full analysis can take minutes.

### Solution
Add `--only-changed-since <commit-ref>` flag that uses `git diff --name-only` to filter files before analysis, providing 10-100x speedup on incremental runs.

### Implementation Steps

#### Task 2.1: Add CLI Argument
**File:** `src/cli.rs`
**Location:** After the threshold arguments (around line 135)

```rust
/// Only analyze files changed since the specified git commit
#[arg(
    long,
    value_name = "COMMIT",
    help = "Only analyze files changed since this commit (e.g., HEAD~1, main, abc123)"
)]
pub only_changed_since: Option<String>,
```

**Update Default impl:**
```rust
only_changed_since: None,
```

#### Task 2.2: Create Git Integration Module
**File:** `src/analyzer/git.rs` (NEW FILE)

```rust
//! Git integration for analyzing only changed files
//!
//! This module provides functionality to integrate with git repositories,
//! allowing the analyzer to focus only on files that have changed since
//! a specific commit reference.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AnalyzerError, Result};

/// Get list of files changed since a specific commit
///
/// Executes `git diff --name-only <commit_ref>` and returns the list of
/// changed file paths as absolute paths.
///
/// # Arguments
/// * `repo_path` - Path to the git repository root (or any subdirectory)
/// * `commit_ref` - Git commit reference (e.g., "HEAD~1", "main", "abc123")
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - List of changed file paths (absolute)
/// * `Err(AnalyzerError)` - If git command fails or not in a git repo
///
/// # Examples
/// ```ignore
/// let changed = get_changed_files("./", "HEAD~1")?;
/// println!("Changed files: {:?}", changed);
/// ```
pub fn get_changed_files<P: AsRef<Path>>(
    repo_path: P,
    commit_ref: &str,
) -> Result<Vec<PathBuf>> {
    let repo_path = repo_path.as_ref();

    // Get the repository root first
    let repo_root = get_repo_root(repo_path)?;

    // Execute git diff --name-only to get changed files
    let output = Command::new("git")
        .args(["diff", "--name-only", commit_ref])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| {
            AnalyzerError::validation_error(format!(
                "Failed to execute git command: {}. Is git installed and in PATH?",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AnalyzerError::validation_error(format!(
            "Git diff failed for ref '{}': {}",
            commit_ref,
            stderr.trim()
        )));
    }

    // Also get staged files (git diff --cached)
    let staged_output = Command::new("git")
        .args(["diff", "--name-only", "--cached"])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| {
            AnalyzerError::validation_error(format!(
                "Failed to get staged files: {}",
                e
            ))
        })?;

    // Parse output into file paths
    let stdout = String::from_utf8_lossy(&output.stdout);
    let staged_stdout = String::from_utf8_lossy(&staged_output.stdout);

    let mut changed_files: Vec<PathBuf> = stdout
        .lines()
        .chain(staged_stdout.lines())
        .filter(|line| !line.is_empty())
        .map(|line| repo_root.join(line))
        .filter(|path| path.exists()) // Only include files that still exist
        .collect();

    // Deduplicate (a file could be in both diff and cached)
    changed_files.sort();
    changed_files.dedup();

    Ok(changed_files)
}

/// Get the git repository root path
///
/// Executes `git rev-parse --show-toplevel` to find the repository root.
///
/// # Arguments
/// * `path` - Any path within the git repository
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to repository root
/// * `Err(AnalyzerError)` - If not in a git repository
pub fn get_repo_root<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();

    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .map_err(|e| {
            AnalyzerError::validation_error(format!(
                "Failed to execute git command: {}. Is git installed?",
                e
            ))
        })?;

    if !output.status.success() {
        return Err(AnalyzerError::validation_error(format!(
            "Not a git repository: {}. The --only-changed-since flag requires a git repository.",
            path.display()
        )));
    }

    let root = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(PathBuf::from(root))
}

/// Check if a path is inside a git repository
pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
    get_repo_root(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_git_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .expect("Failed to init git repo");

        // Configure git user (required for commits)
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(root)
            .output()
            .expect("Failed to configure git email");

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(root)
            .output()
            .expect("Failed to configure git name");

        // Create initial file and commit
        fs::write(root.join("initial.rs"), "fn main() {}").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .expect("Failed to git add");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(root)
            .output()
            .expect("Failed to git commit");

        dir
    }

    #[test]
    fn test_get_repo_root() {
        let repo = create_git_repo();
        let root = repo.path();

        let result = get_repo_root(root);
        assert!(result.is_ok());

        let repo_root = result.unwrap();
        assert!(repo_root.exists());
    }

    #[test]
    fn test_get_repo_root_not_a_repo() {
        let dir = TempDir::new().unwrap();
        let result = get_repo_root(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_git_repository() {
        let repo = create_git_repo();
        assert!(is_git_repository(repo.path()));

        let not_repo = TempDir::new().unwrap();
        assert!(!is_git_repository(not_repo.path()));
    }

    #[test]
    fn test_get_changed_files_with_staged() {
        let repo = create_git_repo();
        let root = repo.path();

        // Create a new file and stage it
        fs::write(root.join("new_file.rs"), "fn new() {}").unwrap();

        Command::new("git")
            .args(["add", "new_file.rs"])
            .current_dir(root)
            .output()
            .expect("Failed to stage file");

        // Get changed files since HEAD
        let changed = get_changed_files(root, "HEAD").unwrap();

        // new_file.rs should be in the list
        assert!(
            changed.iter().any(|p| p.file_name().unwrap() == "new_file.rs"),
            "Staged file should be in changed list"
        );
    }

    #[test]
    fn test_get_changed_files_invalid_ref() {
        let repo = create_git_repo();
        let result = get_changed_files(repo.path(), "invalid_ref_that_does_not_exist");
        assert!(result.is_err());
    }
}
```

#### Task 2.3: Update Module Declarations
**File:** `src/analyzer/mod.rs`
**Location:** After line 12

Add:
```rust
pub mod git;
pub use git::{get_changed_files, get_repo_root, is_git_repository};
```

#### Task 2.4: Integrate with AnalyzerEngine
**File:** `src/analyzer/mod.rs`
**Location:** Modify `analyze_project` method (around line 73)

```rust
/// Analyze a project directory and return comprehensive results
pub fn analyze_project<P: AsRef<Path>>(
    &mut self,
    target_path: P,
    cli_args: &CliArgs,
) -> Result<AnalysisReport> {
    let target_path = target_path.as_ref();

    if self.show_progress {
        println!("Starting analysis of: {}", target_path.display());
    }

    // Step 1: Discover files (with optional git filtering)
    let (files, walk_stats) = if let Some(ref commit_ref) = cli_args.only_changed_since {
        self.discover_git_changed_files(target_path, commit_ref)?
    } else {
        self.file_walker.discover_files(target_path)?
    };

    // ... rest of method unchanged
}

/// Discover only files changed since a git commit
fn discover_git_changed_files<P: AsRef<Path>>(
    &self,
    target_path: P,
    commit_ref: &str,
) -> Result<(Vec<PathBuf>, WalkStats)> {
    let target_path = target_path.as_ref();

    if self.show_progress {
        println!("Git mode: analyzing files changed since '{}'", commit_ref);
    }

    // Get changed files from git
    let changed_files = git::get_changed_files(target_path, commit_ref)?;

    if self.show_progress {
        println!("Git reports {} changed files", changed_files.len());
    }

    // Filter through the normal language/size filters
    let filtered_files: Vec<PathBuf> = changed_files
        .into_iter()
        .filter(|f| self.file_parser.can_parse(f))
        .collect();

    let stats = WalkStats {
        files_found: filtered_files.len(),
        total_entries_scanned: filtered_files.len(),
        ..Default::default()
    };

    if self.show_progress {
        println!(
            "After filtering: {} files to analyze",
            filtered_files.len()
        );
    }

    Ok((filtered_files, stats))
}
```

#### Task 2.5: Add Integration Tests
**File:** `tests/integration_tests.rs`
**Location:** Add new test module

```rust
mod git_integration {
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_git_repo_with_rust_files() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Initialize repo
        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(root)
            .output()
            .unwrap();

        // Create and commit initial file
        fs::write(
            root.join("old_file.rs"),
            "fn old() {\n    println!(\"old\");\n}\n",
        )
        .unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(root)
            .output()
            .unwrap();

        // Add new file (staged but not committed)
        fs::write(
            root.join("new_file.rs"),
            "fn new() {\n    println!(\"new\");\n    if true {\n        println!(\"complex\");\n    }\n}\n",
        )
        .unwrap();

        Command::new("git")
            .args(["add", "new_file.rs"])
            .current_dir(root)
            .output()
            .unwrap();

        dir
    }

    #[test]
    fn test_only_changed_since_analyzes_only_new_files() {
        let repo = create_git_repo_with_rust_files();

        // Run analysis with --only-changed-since HEAD
        let output = Command::new(env!("CARGO_BIN_EXE_code-analyzer"))
            .args(["--only-changed-since", "HEAD", "--output", "json"])
            .current_dir(repo.path())
            .output()
            .expect("Failed to run analyzer");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should succeed
        assert!(
            output.status.success(),
            "Analysis should succeed. stderr: {}",
            stderr
        );

        // Output should mention new_file.rs
        assert!(
            stdout.contains("new_file.rs") || stderr.contains("new_file.rs"),
            "Should analyze new_file.rs"
        );

        // Should NOT analyze old_file.rs (not in changes)
        // (This is harder to verify from output alone)
    }

    #[test]
    fn test_only_changed_since_fails_gracefully_outside_git() {
        let non_git_dir = TempDir::new().unwrap();

        // Create a Rust file but don't init git
        fs::write(
            non_git_dir.path().join("file.rs"),
            "fn main() {}",
        )
        .unwrap();

        let output = Command::new(env!("CARGO_BIN_EXE_code-analyzer"))
            .args(["--only-changed-since", "HEAD"])
            .current_dir(non_git_dir.path())
            .output()
            .expect("Failed to run analyzer");

        // Should fail with clear error message
        assert!(!output.status.success());

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("git repository") || stderr.contains("Not a git"),
            "Should explain that path is not a git repository"
        );
    }
}
```

### Acceptance Criteria
- [ ] `code-analyzer --only-changed-since HEAD` analyzes only uncommitted changes
- [ ] `code-analyzer --only-changed-since HEAD~3` analyzes files changed in last 3 commits
- [ ] `code-analyzer --only-changed-since main` analyzes files changed since main branch
- [ ] Clear error message when not in a git repository
- [ ] Clear error message when git is not installed
- [ ] Clear error message for invalid commit references
- [ ] Files deleted since the commit are gracefully skipped
- [ ] Performance: 10x+ speedup measurable on incremental analysis

---

## Phase 3: Enhanced JSON Output Options (Medium Priority)

### Problem Statement
The current JSON output always exports the full `AnalysisReport` structure. For dashboards or APIs that only need file metrics or project summary, this creates unnecessary payload size and parsing overhead.

### Solution
Expose the existing `export_files_only()` and `export_summary_only()` functions via new CLI output format options.

### Implementation Steps

#### Task 3.1: Extend OutputFormat Enum
**File:** `src/cli.rs`
**Location:** Line 153

```rust
/// Output format options
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum OutputFormat {
    /// Terminal table output only
    Table,
    /// JSON output only (full report)
    Json,
    /// Both terminal table and JSON output
    Both,
    /// JSON with only file analysis data (no summary/config)
    #[value(name = "json-files-only")]
    JsonFilesOnly,
    /// JSON with only project summary (no individual files)
    #[value(name = "json-summary-only")]
    JsonSummaryOnly,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Both => write!(f, "both"),
            OutputFormat::JsonFilesOnly => write!(f, "json-files-only"),
            OutputFormat::JsonSummaryOnly => write!(f, "json-summary-only"),
        }
    }
}
```

#### Task 3.2: Update should_output Methods
**File:** `src/cli.rs`
**Location:** Around line 244

```rust
/// Check if JSON output should be generated
pub fn should_output_json(&self) -> bool {
    matches!(
        self.output,
        OutputFormat::Json
            | OutputFormat::Both
            | OutputFormat::JsonFilesOnly
            | OutputFormat::JsonSummaryOnly
    ) || self.json_only
}

/// Check if terminal output should be displayed
pub fn should_output_terminal(&self) -> bool {
    !self.json_only
        && matches!(self.output, OutputFormat::Table | OutputFormat::Both)
}
```

#### Task 3.3: Update OutputManager
**File:** `src/output/mod.rs`

Update `generate_output` method:
```rust
pub fn generate_output(&self, report: &AnalysisReport, cli_args: &CliArgs) -> Result<()> {
    match cli_args.output {
        OutputFormat::Table => {
            self.generate_terminal_output(report, cli_args)?;
        }
        OutputFormat::Json => {
            self.generate_json_output(report, cli_args)?;
        }
        OutputFormat::Both => {
            self.generate_terminal_output(report, cli_args)?;
            self.generate_json_output(report, cli_args)?;
        }
        OutputFormat::JsonFilesOnly => {
            let path = cli_args.json_output_path();
            self.json_exporter.export_files_only(&report.files, &path)?;
            if cli_args.verbose {
                println!("Files-only JSON written to: {}", path.display());
            }
        }
        OutputFormat::JsonSummaryOnly => {
            let path = cli_args.json_output_path();
            self.json_exporter.export_summary_only(&report.summary, &path)?;
            if cli_args.verbose {
                println!("Summary-only JSON written to: {}", path.display());
            }
        }
    }
    Ok(())
}
```

#### Task 3.4: Add Tests
**File:** `tests/integration_tests.rs`

```rust
#[test]
fn test_json_files_only_output() {
    let test_dir = create_test_project();
    let output_file = test_dir.path().join("output.json");

    let output = Command::new(env!("CARGO_BIN_EXE_code-analyzer"))
        .args([
            "--output", "json-files-only",
            "--output-file", output_file.to_str().unwrap(),
        ])
        .current_dir(test_dir.path())
        .output()
        .expect("Failed to run analyzer");

    assert!(output.status.success());

    let content = fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Should be an array of file objects
    assert!(json.is_array(), "Output should be an array");

    // Each item should have file analysis fields
    if let Some(first) = json.as_array().and_then(|a| a.first()) {
        assert!(first.get("path").is_some());
        assert!(first.get("language").is_some());
        assert!(first.get("lines_of_code").is_some());
    }
}

#[test]
fn test_json_summary_only_output() {
    let test_dir = create_test_project();
    let output_file = test_dir.path().join("summary.json");

    let output = Command::new(env!("CARGO_BIN_EXE_code-analyzer"))
        .args([
            "--output", "json-summary-only",
            "--output-file", output_file.to_str().unwrap(),
        ])
        .current_dir(test_dir.path())
        .output()
        .expect("Failed to run analyzer");

    assert!(output.status.success());

    let content = fs::read_to_string(&output_file).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Should be a summary object
    assert!(json.is_object(), "Output should be an object");
    assert!(json.get("total_files").is_some());
    assert!(json.get("total_lines").is_some());
    assert!(json.get("language_breakdown").is_some());
}
```

### Acceptance Criteria
- [ ] `--output json-files-only` produces JSON array of FileAnalysis objects
- [ ] `--output json-summary-only` produces JSON ProjectSummary object
- [ ] File size is 30-50% smaller than full report for each mode
- [ ] Existing `--output json` behavior unchanged
- [ ] `--help` shows new options with descriptions

---

## Phase 4: CI/CD Integration Enhancements (Medium Priority)

### Problem Statement
CI/CD pipelines need programmatic ways to detect code quality issues. Currently, the tool always exits with code 0 on successful analysis, even when refactoring candidates are found.

### Solution
Add `--ci` flag that exits with code 2 when refactoring candidates exceed a threshold.

### Implementation Steps

#### Task 4.1: Add CI Mode Flags
**File:** `src/cli.rs`

```rust
/// CI mode: exit with code 2 if refactoring candidates found
#[arg(
    long,
    help = "CI mode: exit code 2 if refactoring candidates exceed threshold"
)]
pub ci: bool,

/// Maximum allowed candidates in CI mode before failing
#[arg(
    long,
    default_value_t = 0,
    help = "Max allowed refactoring candidates in CI mode (0 = any triggers exit code 2)"
)]
pub ci_max_candidates: usize,
```

#### Task 4.2: Update main.rs with Exit Codes
**File:** `src/main.rs`

```rust
use code_analyzer::{run_analysis, CliArgs, identify_refactoring_candidates, RefactoringThresholds};
use clap::Parser;

// Exit codes for CI integration
const EXIT_SUCCESS: i32 = 0;
const EXIT_ERROR: i32 = 1;
const EXIT_CANDIDATES_EXCEEDED: i32 = 2;

fn main() {
    let args = CliArgs::parse();

    // Store CI settings before moving args
    let ci_mode = args.ci;
    let ci_max = args.ci_max_candidates;
    let thresholds = RefactoringThresholds::from_cli(&args);

    match code_analyzer::run_analysis_returning_report(args) {
        Ok(report) => {
            if ci_mode {
                let candidates = identify_refactoring_candidates(
                    &report.files,
                    &thresholds,
                );

                if candidates.len() > ci_max {
                    eprintln!(
                        "CI check failed: {} refactoring candidates found (max allowed: {})",
                        candidates.len(),
                        ci_max
                    );
                    std::process::exit(EXIT_CANDIDATES_EXCEEDED);
                }
            }
            std::process::exit(EXIT_SUCCESS);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(EXIT_ERROR);
        }
    }
}
```

#### Task 4.3: Add run_analysis_returning_report Function
**File:** `src/lib.rs`

```rust
/// Run analysis and return the report (for programmatic use)
pub fn run_analysis_returning_report(args: CliArgs) -> Result<AnalysisReport> {
    args.validate()?;

    let mut analyzer = AnalyzerEngine::from_cli_args(&args)?;
    let report = analyzer.analyze_project(args.target_path(), &args)?;

    // Generate output
    if args.compact {
        output::display_compact_table(&report.files, args.sort, args.limit);
    } else {
        let output_manager = OutputManager::from_cli_args(&args);
        output_manager.generate_output(&report, &args)?;
    }

    Ok(report)
}
```

### Acceptance Criteria
- [ ] `code-analyzer --ci` exits with code 2 if any refactoring candidates found
- [ ] `code-analyzer --ci --ci-max-candidates 5` allows up to 5 candidates
- [ ] Exit code 0 when candidates <= threshold
- [ ] Exit code 1 on analysis error
- [ ] Clear message printed to stderr on CI failure

---

## Phase 5: Minor Improvements (Low Priority)

### Task 5.1: Enhanced Walk Stats Display
**File:** `src/output/terminal.rs`

Add method to display walker statistics in the report:
```rust
/// Display file discovery statistics
pub fn display_walk_stats(&self, stats: &crate::analyzer::walker::WalkStats) {
    if stats.files_skipped_size > 0
        || stats.files_skipped_language > 0
        || stats.files_skipped_hidden > 0
    {
        println!();
        println!("File Discovery:");
        println!("├─ Analyzed: {}", stats.files_found);
        if stats.files_skipped_size > 0 {
            println!("├─ Skipped (too large): {}", stats.files_skipped_size);
        }
        if stats.files_skipped_language > 0 {
            println!("├─ Skipped (unsupported language): {}", stats.files_skipped_language);
        }
        if stats.files_skipped_hidden > 0 {
            println!("├─ Skipped (hidden): {}", stats.files_skipped_hidden);
        }
        println!("└─ Directories scanned: {}", stats.directories_scanned);
    }
}
```

### Task 5.2: Remove Deprecated display_top_files
**File:** `src/output/terminal.rs`

Remove the `#[deprecated]` method `display_top_files` (lines 367-409) after confirming no active call sites remain.

### Task 5.3: Windows Path Normalization
**File:** `src/output/terminal.rs`

Update `format_file_path` method to normalize Windows backslashes:
```rust
fn format_file_path(&self, path: &std::path::Path) -> String {
    let display_path = if let Some(ref base) = self.base_path {
        path.strip_prefix(base)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    let mut path_str = display_path.display().to_string();

    // Normalize Windows backslashes to forward slashes for consistent display
    #[cfg(target_os = "windows")]
    {
        path_str = path_str.replace('\\', "/");
    }

    const MAX_PATH_LENGTH: usize = 50;
    if path_str.len() > MAX_PATH_LENGTH {
        let start = path_str.len() - MAX_PATH_LENGTH + 3;
        format!("...{}", &path_str[start..])
    } else {
        path_str
    }
}
```

---

## Implementation Schedule

### Sprint 1: Custom Thresholds (Phase 1) - Critical
**Estimated effort:** 2-3 hours

- [ ] Task 1.1: Add CLI arguments
- [ ] Task 1.2: Create RefactoringThresholds struct
- [ ] Task 1.3: Update identify_refactoring_candidates signature
- [ ] Task 1.4: Update all call sites
- [ ] Task 1.5: Add unit tests
- [ ] Validation: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`

### Sprint 2: Git Integration (Phase 2) - High Priority
**Estimated effort:** 3-4 hours

- [ ] Task 2.1: Add CLI argument
- [ ] Task 2.2: Create git.rs module
- [ ] Task 2.3: Update module declarations
- [ ] Task 2.4: Integrate with AnalyzerEngine
- [ ] Task 2.5: Add integration tests
- [ ] Validation: Manual test with real git repository

### Sprint 3: JSON & CI Enhancements (Phases 3-4) - Medium Priority
**Estimated effort:** 2-3 hours

- [ ] Task 3.1-3.4: Enhanced JSON output options
- [ ] Task 4.1-4.3: CI mode with exit codes
- [ ] Add integration tests for both
- [ ] Validation: Test in mock CI environment

### Sprint 4: Polish & Documentation (Phase 5)
**Estimated effort:** 1-2 hours

- [ ] Task 5.1: Walk stats display
- [ ] Task 5.2: Remove deprecated methods
- [ ] Task 5.3: Windows path normalization
- [ ] Update README.md with new features
- [ ] Update CHANGELOG.md

---

## Testing Strategy

### Unit Tests
- Each new function must have corresponding unit tests
- Minimum 80% code coverage for new code
- Test edge cases: empty inputs, invalid values, boundary conditions

### Integration Tests
- CLI flag combinations
- Git integration with real repositories
- Output format validation
- CI mode exit codes

### Manual Testing Checklist
- [ ] Run on this codebase: `cargo run -- .`
- [ ] Run with custom thresholds: `cargo run -- --max-cc 10 --max-loc 200`
- [ ] Run in git mode: `cargo run -- --only-changed-since HEAD~5`
- [ ] Run in CI mode: `cargo run -- --ci`
- [ ] Verify JSON outputs with `jq`
- [ ] Test on Windows (if available)

---

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Git not installed | Medium | Low | Clear error message, graceful degradation |
| Breaking identify_refactoring_candidates API | High | Medium | Maintain backward-compatible overload |
| Performance regression | Medium | Low | Benchmark before/after |
| Windows path edge cases | Low | Medium | Conditional compilation, CI testing |

---

## Success Metrics

1. **Custom Thresholds**:
   - Users can configure all 4 thresholds via CLI
   - Default behavior unchanged (backward compatible)

2. **Git Integration**:
   - 10x+ speedup on incremental analysis (measured)
   - Clear error messages for edge cases

3. **JSON Output**:
   - 30%+ reduction in output size for specialized modes
   - Clean parsing with standard JSON tools

4. **CI Mode**:
   - Correct exit codes for pipeline integration
   - Works with all major CI systems (GitHub Actions, GitLab CI, Jenkins)

5. **Quality**:
   - Zero regressions (all existing tests pass)
   - New code has >80% test coverage
   - `cargo clippy -- -D warnings` clean

---

## Documentation Updates Required

- [ ] README.md: Document new CLI flags
- [ ] README.md: Add CI/CD integration examples
- [ ] README.md: Add git integration usage
- [ ] `--help` text for all new flags
- [ ] CHANGELOG.md entry for v3.0
