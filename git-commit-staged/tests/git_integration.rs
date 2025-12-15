//! Git integration tests for git-commit-staged
//!
//! These tests call git_commit_staged in-process (git subprocess, self in-process)
//! allowing inspection of Rust types and error variants.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use git_commit_staged::git_commit_staged;

/// Helper to run git commands in a directory
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute git");

    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Create a test repo with an initial commit
fn setup_repo() -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let dir = tmp.path();

    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@test.com"]);
    git(dir, &["config", "user.name", "Test User"]);

    // Initial commit
    fs::write(dir.join("README.md"), "# Test Repo\n").unwrap();
    git(dir, &["add", "README.md"]);
    git(dir, &["commit", "-m", "Initial commit"]);

    tmp
}

#[test]
fn git_status_clean_after_commit() {
    // Verifies that git status shows committed file as clean (not staged).
    // Tests whether skipping unstage_paths causes stale index metadata issues.
    let tmp = setup_repo();
    let dir = tmp.path();

    // Stage a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/file.rs"), "// content\n").unwrap();
    git(dir, &["add", "src/file.rs"]);

    // Commit via git-commit-staged
    git_commit_staged(
        &[PathBuf::from("src/file.rs")],
        "Commit file",
        dir,
        false,
    )
    .expect("commit should succeed");

    // git status --porcelain should show nothing for this file
    let status = git(dir, &["status", "--porcelain"]);
    assert!(
        !status.contains("src/file.rs"),
        "committed file should not appear in git status: {}",
        status
    );

    // git diff --cached should be empty
    let diff_cached = git(dir, &["diff", "--cached", "--name-only"]);
    assert!(
        diff_cached.trim().is_empty(),
        "git diff --cached should be empty: {}",
        diff_cached
    );
}

#[test]
fn errors_on_merge_conflict_with_index_error() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create a file on main
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/conflict.rs"), "// main version\n").unwrap();
    git(dir, &["add", "src/conflict.rs"]);
    git(dir, &["commit", "-m", "Add file on main"]);

    // Create a branch and modify
    git(dir, &["checkout", "-b", "feature"]);
    fs::write(dir.join("src/conflict.rs"), "// feature version\n").unwrap();
    git(dir, &["add", "src/conflict.rs"]);
    git(dir, &["commit", "-m", "Modify on feature"]);

    // Go back to main and make conflicting change
    git(dir, &["checkout", "main"]);
    fs::write(dir.join("src/conflict.rs"), "// conflicting main version\n").unwrap();
    git(dir, &["add", "src/conflict.rs"]);
    git(dir, &["commit", "-m", "Conflicting change on main"]);

    // Merge feature - will conflict
    let merge_output = Command::new("git")
        .args(["merge", "feature"])
        .current_dir(dir)
        .output()
        .expect("failed to run git merge");
    assert!(!merge_output.status.success(), "merge should conflict");

    // Verify we have conflicts
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.contains("UU"), "should have unmerged files: {status}");

    // Call git_commit_staged in-process
    let result = git_commit_staged(
        &[PathBuf::from("src")],
        "Should fail",
        dir,
        false,
    );

    // Should fail
    let err = result.expect_err("should fail on merge conflict");

    // Downcast to find the git2 error
    let git2_err = err
        .chain()
        .find_map(|e| e.downcast_ref::<git2::Error>())
        .expect("should contain git2::Error in chain");

    assert_eq!(
        git2_err.class(),
        git2::ErrorClass::Index,
        "error should be Index class, got: {:?}",
        git2_err
    );
}
