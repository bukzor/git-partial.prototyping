//! Git integration tests for git-commit-staged
//!
//! These tests call `git_commit_staged` in-process (git subprocess, self in-process)
//! allowing inspection of Rust types and error variants.
//!
//! Tests that require CLI-specific behavior (argument parsing, -C flag, -n flag,
//! exit codes, stderr formatting) remain in integration.rs.

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
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

/// Create a test repo without any commits (empty repo)
fn setup_empty_repo() -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let dir = tmp.path();

    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@test.com"]);
    git(dir, &["config", "user.name", "Test User"]);

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
    git_commit_staged(&[PathBuf::from("src/file.rs")], "Commit file", dir, false)
        .expect("commit should succeed");

    // git status --porcelain should show nothing for this file
    let status = git(dir, &["status", "--porcelain"]);
    assert!(
        !status.contains("src/file.rs"),
        "committed file should not appear in git status: {status}"
    );

    // git diff --cached should be empty
    let diff_cached = git(dir, &["diff", "--cached", "--name-only"]);
    assert!(
        diff_cached.trim().is_empty(),
        "git diff --cached should be empty: {diff_cached}"
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
    assert!(
        status.contains("UU"),
        "should have unmerged files: {status}"
    );

    // Call git_commit_staged in-process
    let result = git_commit_staged(&[PathBuf::from("src")], "Should fail", dir, false);

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
        "error should be Index class, got: {git2_err:?}"
    );
}

// =============================================================================
// Happy path tests (ported from integration.rs)
// =============================================================================

#[test]
fn commits_staged_file_at_specified_path() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and stage a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Commit it
    let result = git_commit_staged(&[PathBuf::from("src")], "Add main.rs", dir, false)
        .expect("commit should succeed");

    assert!(result.commit_oid.is_some());

    // Verify commit exists
    let log = git(dir, &["log", "--oneline", "-1"]);
    assert!(log.contains("Add main.rs"));

    // Verify file is in the commit
    let show = git(dir, &["show", "--name-only", "--format="]);
    assert!(show.contains("src/main.rs"));
}

#[test]
fn only_commits_staged_not_working_copy() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and stage a file with content "v1"
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "// v1\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Modify working copy to "v2" (not staged)
    fs::write(dir.join("src/main.rs"), "// v2\n").unwrap();

    // Commit using our tool
    git_commit_staged(&[PathBuf::from("src")], "Add v1", dir, false)
        .expect("commit should succeed");

    // Verify the committed content is v1, not v2
    let content = git(dir, &["show", "HEAD:src/main.rs"]);
    assert_eq!(content.trim(), "// v1");

    // Working copy should still have v2
    let working = fs::read_to_string(dir.join("src/main.rs")).unwrap();
    assert_eq!(working.trim(), "// v2");
}

#[test]
fn preserves_other_staged_files() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Stage two files in different directories
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.join("tests/test.rs"), "#[test] fn it_works() {}\n").unwrap();
    git(dir, &["add", "src/main.rs", "tests/test.rs"]);

    // Commit only src/
    git_commit_staged(&[PathBuf::from("src")], "Add main.rs only", dir, false)
        .expect("commit should succeed");

    // Verify only main.rs was committed
    let show = git(dir, &["show", "--name-only", "--format="]);
    assert!(show.contains("src/main.rs"));
    assert!(!show.contains("tests/test.rs"));

    // Verify tests/test.rs is still staged
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.contains("A  tests/test.rs"));
}

#[test]
fn rejects_escaping_path() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Try to commit a path that escapes the repo
    let result = git_commit_staged(&[PathBuf::from("../outside")], "Should fail", dir, false);

    let err = result.expect_err("should fail on escaping path");
    assert!(
        err.to_string().contains("escapes scope"),
        "error should mention escaping scope: {err}"
    );
}

#[test]
#[cfg(unix)]
fn commits_staged_mode_change() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a non-executable file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/script.sh"), "#!/bin/bash\necho hello\n").unwrap();
    git(dir, &["add", "src/script.sh"]);
    git(dir, &["commit", "-m", "Add script"]);

    // chmod +x and stage
    Command::new("chmod")
        .args(["+x", "src/script.sh"])
        .current_dir(dir)
        .status()
        .unwrap();
    git(dir, &["add", "src/script.sh"]);

    let status = git(dir, &["status", "--porcelain"]);
    assert_eq!(status, "M  src/script.sh\n");

    // Commit the mode change
    git_commit_staged(
        &[PathBuf::from("src")],
        "Make script executable",
        dir,
        false,
    )
    .expect("should commit mode change");

    // Verify the commit happened
    let log = git(dir, &["log", "--oneline", "-1"]);
    assert!(log.contains("Make script executable"));

    // Verify the executable bit was actually committed
    git(dir, &["checkout", "HEAD", "--", "src/script.sh"]);
    let metadata = std::fs::metadata(dir.join("src/script.sh")).unwrap();
    let mode = metadata.permissions().mode();
    assert_eq!(
        mode & 0o111,
        0o111,
        "file should be executable, mode: {mode:o}"
    );
}

#[test]
#[cfg(unix)]
fn commits_staged_typechange() {
    use std::os::unix::fs::symlink;

    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a regular file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/link.txt"), "original content\n").unwrap();
    git(dir, &["add", "src/link.txt"]);
    git(dir, &["commit", "-m", "Add regular file"]);

    // Replace with symlink (typechange)
    fs::remove_file(dir.join("src/link.txt")).unwrap();
    symlink("../README.md", dir.join("src/link.txt")).unwrap();
    git(dir, &["add", "src/link.txt"]);

    let status = git(dir, &["status", "--porcelain"]);
    assert!(
        status.contains("T  src/link.txt"),
        "should show typechange: {status}"
    );

    // Commit the typechange
    git_commit_staged(&[PathBuf::from("src")], "Change to symlink", dir, false)
        .expect("should commit typechange");

    // Verify it's a symlink in HEAD
    let ls_tree = git(dir, &["ls-tree", "HEAD", "src/link.txt"]);
    assert!(
        ls_tree.contains("120000"),
        "should be symlink mode (120000): {ls_tree}"
    );
}

#[test]
fn commits_staged_deletion() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/obsolete.rs"), "// to be deleted\n").unwrap();
    git(dir, &["add", "src/obsolete.rs"]);
    git(dir, &["commit", "-m", "Add obsolete file"]);

    // Stage deletion
    git(dir, &["rm", "src/obsolete.rs"]);

    let status = git(dir, &["status", "--porcelain"]);
    assert_eq!(status, "D  src/obsolete.rs\n");

    // Commit the deletion
    git_commit_staged(&[PathBuf::from("src")], "Remove obsolete file", dir, false)
        .expect("should commit deletion");

    // Verify file is gone from HEAD
    let show = git(dir, &["show", "--name-status", "--format="]);
    assert!(show.contains("D\tsrc/obsolete.rs"));
}

#[test]
fn commits_staged_rename() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/old_name.rs"), "// content\n").unwrap();
    git(dir, &["add", "src/old_name.rs"]);
    git(dir, &["commit", "-m", "Add old_name.rs"]);

    // Rename it
    git(dir, &["mv", "src/old_name.rs", "src/new_name.rs"]);

    let status = git(dir, &["status", "--porcelain"]);
    assert_eq!(status, "R  src/old_name.rs -> src/new_name.rs\n");

    // Commit specifying the src directory (should get both delete + add)
    git_commit_staged(&[PathBuf::from("src")], "Rename file", dir, false)
        .expect("should commit rename");

    // Verify both old gone and new exists
    let show = git(dir, &["show", "--name-status", "--format="]);
    assert!(
        show.contains("src/new_name.rs"),
        "new file should exist: {show}"
    );
    // Old file should be gone from HEAD
    let ls = git(dir, &["ls-tree", "HEAD", "src/"]);
    assert!(!ls.contains("old_name.rs"), "old file should be gone: {ls}");
}

#[test]
fn commits_rename_with_detection_enabled() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/old_name.rs"), "// content\n").unwrap();
    git(dir, &["add", "src/old_name.rs"]);
    git(dir, &["commit", "-m", "Add old_name.rs"]);

    // Enable rename detection in git config
    git(dir, &["config", "diff.renames", "true"]);

    // Rename it
    git(dir, &["mv", "src/old_name.rs", "src/new_name.rs"]);

    let status = git(dir, &["status", "--porcelain"]);
    assert_eq!(status, "R  src/old_name.rs -> src/new_name.rs\n");

    // Commit specifying the src directory
    git_commit_staged(&[PathBuf::from("src")], "Rename file", dir, false)
        .expect("should commit rename with detection enabled");

    // Verify both old gone and new exists
    let show = git(dir, &["show", "--name-status", "--format="]);
    assert!(
        show.contains("src/new_name.rs"),
        "new file should exist: {show}"
    );
    // Old file should be gone from HEAD - this is the critical check
    let ls = git(dir, &["ls-tree", "-r", "HEAD"]);
    assert!(
        !ls.contains("old_name.rs"),
        "old file should be deleted but persists: {ls}"
    );

    // Extra validation: check commit tree contents directly
    let commit_tree = git(dir, &["rev-parse", "HEAD^{tree}"]);
    let tree_contents = git(dir, &["ls-tree", "-r", commit_tree.trim()]);
    assert!(
        !tree_contents.contains("old_name.rs"),
        "committed tree should not have old_name.rs: {tree_contents}"
    );
}

// =============================================================================
// Error tests (ported from integration.rs)
// =============================================================================

#[test]
fn errors_on_empty_repo() {
    let tmp = setup_empty_repo();
    let dir = tmp.path();

    // Create and stage a file but don't commit
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Tool should error - no HEAD exists
    let result = git_commit_staged(&[PathBuf::from("src")], "First commit", dir, false);

    let err = result.expect_err("should fail on empty repo");
    let err_str = err.to_string().to_lowercase();
    assert!(err_str.contains("head"), "error should mention HEAD: {err}");
}

#[test]
fn errors_when_no_staged_changes() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and COMMIT a file (not just staged - fully committed)
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);
    git(dir, &["commit", "-m", "Add main.rs"]);

    // Now src/main.rs exists in HEAD and index, but is NOT staged (no changes)
    // The tool should error, not create an empty commit
    let result = git_commit_staged(&[PathBuf::from("src")], "Nothing staged", dir, false);

    let err = result.expect_err("should fail when no staged changes");
    assert!(
        err.to_string().contains("no staged changes"),
        "error should mention no staged changes: {err}"
    );
}

#[test]
fn committed_files_unstaged() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Stage two files
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.join("tests/test.rs"), "#[test] fn test() {}\n").unwrap();
    git(dir, &["add", "src/main.rs", "tests/test.rs"]);

    // Verify both are staged
    let status_before = git(dir, &["status", "--porcelain"]);
    assert!(status_before.contains("A  src/main.rs"));
    assert!(status_before.contains("A  tests/test.rs"));

    // Commit only src/
    git_commit_staged(&[PathBuf::from("src")], "Add main.rs", dir, false)
        .expect("commit should succeed");

    // Verify src/main.rs is no longer staged but tests/test.rs still is
    let status_after = git(dir, &["status", "--porcelain"]);
    assert!(
        !status_after.contains("src/main.rs"),
        "committed file should be unstaged: {status_after}"
    );
    assert!(
        status_after.contains("A  tests/test.rs"),
        "uncommitted file should still be staged: {status_after}"
    );
}
