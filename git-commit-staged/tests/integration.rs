//! CLI integration tests for git-commit-staged
//!
//! These tests run git-commit-staged as a subprocess to test CLI-specific behavior:
//! argument parsing, -C flag, -n flag, exit codes, stderr formatting.
//!
//! Core logic tests are in `git_integration.rs` where they run in-process.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

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

/// Helper to run our binary
fn git_commit_staged(dir: &Path, args: &[&str]) -> std::process::Output {
    let binary = env!("CARGO_BIN_EXE_git-commit-staged");
    Command::new(binary)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute git-commit-staged")
}

/// Create a test repo with an initial commit
fn setup_repo() -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let dir = tmp.path();

    git(dir, &["init", "-b", "main"]);
    git(dir, &["config", "user.email", "test@test.com"]);
    git(dir, &["config", "user.name", "Test User"]);

    // Initial commit
    fs::write(dir.join("README.md"), "# Test Repo\n").unwrap();
    git(dir, &["add", "README.md"]);
    git(dir, &["commit", "-m", "Initial commit"]);

    tmp
}

#[test]
fn respects_directory_scope() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create nested structure
    fs::create_dir_all(dir.join("pkg/src")).unwrap();
    fs::write(dir.join("pkg/src/lib.rs"), "// lib\n").unwrap();
    git(dir, &["add", "pkg/src/lib.rs"]);

    // Commit from within pkg/ using -C
    let output = git_commit_staged(dir, &["-C", "pkg", "src", "-m", "Add lib.rs"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify commit
    let show = git(dir, &["show", "--name-only", "--format="]);
    assert!(show.contains("pkg/src/lib.rs"));
}

#[test]
fn directory_scope_prevents_escape() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create structure
    fs::create_dir_all(dir.join("pkg/src")).unwrap();
    fs::create_dir_all(dir.join("other")).unwrap();
    fs::write(dir.join("pkg/src/lib.rs"), "// lib\n").unwrap();
    fs::write(dir.join("other/file.rs"), "// other\n").unwrap();
    git(dir, &["add", "."]);

    // Try to escape pkg/ scope using ../
    let output = git_commit_staged(dir, &["-C", "pkg", "../other", "-m", "Should fail"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("escapes scope"));
}

#[test]
fn dry_run_does_not_commit() {
    let tmp = setup_repo();
    let dir = tmp.path();

    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Get commit count before
    let log_before = git(dir, &["rev-list", "--count", "HEAD"]);

    // Dry run
    let output = git_commit_staged(dir, &["-n", "src", "-m", "Dry run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run"));

    // Commit count should be unchanged
    let log_after = git(dir, &["rev-list", "--count", "HEAD"]);
    assert_eq!(log_before, log_after);

    // File should still be staged
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.contains("A  src/main.rs"));
}
