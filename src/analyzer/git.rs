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
pub fn get_changed_files<P: AsRef<Path>>(repo_path: P, commit_ref: &str) -> Result<Vec<PathBuf>> {
    let repo_path = repo_path.as_ref();

    // Get the repository root first
    let repo_root = get_repo_root(repo_path)?;

    // Execute git diff --name-only to get changed files (unstaged + staged vs commit_ref)
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
            AnalyzerError::validation_error(format!("Failed to get staged files: {}", e))
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

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();

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
            changed
                .iter()
                .any(|p| p.file_name().unwrap() == "new_file.rs"),
            "Staged file should be in changed list"
        );
    }

    #[test]
    fn test_get_changed_files_invalid_ref() {
        let repo = create_git_repo();
        let result = get_changed_files(repo.path(), "invalid_ref_that_does_not_exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_changed_files_empty() {
        let repo = create_git_repo();

        // No changes since HEAD - should return empty
        let changed = get_changed_files(repo.path(), "HEAD").unwrap();
        assert!(
            changed.is_empty(),
            "No changes expected immediately after commit"
        );
    }

    #[test]
    fn test_get_changed_files_modified() {
        let repo = create_git_repo();
        let root = repo.path();

        // Modify existing file (don't stage)
        fs::write(
            root.join("initial.rs"),
            "fn main() { println!(\"hello\"); }",
        )
        .unwrap();

        // Get changed files since HEAD
        let changed = get_changed_files(root, "HEAD").unwrap();

        // initial.rs should be in the list
        assert!(
            changed
                .iter()
                .any(|p| p.file_name().unwrap() == "initial.rs"),
            "Modified file should be in changed list"
        );
    }
}
