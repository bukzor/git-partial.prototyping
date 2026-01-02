//! CLI integration tests for git-commit-files
//!
//! These tests run git-commit-files as a subprocess via `git -C <dir> commit-files`
//! to test CLI-specific behavior: staging from working tree, argument parsing, etc.

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

/// Helper to run our binary via `git -C <dir> commit-files`
fn git_commit_files(dir: &Path, args: &[&str]) -> std::process::Output {
    let binary = env!("CARGO_BIN_EXE_git-commit-files");
    let bin_dir = Path::new(binary).parent().unwrap();
    let path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path);

    Command::new("git")
        .arg("-C")
        .arg(dir)
        .arg("commit-files")
        .args(args)
        .env("PATH", new_path)
        .output()
        .expect("failed to execute git commit-files")
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
fn commits_unstaged_working_tree_changes() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create a file but don't stage it
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();

    // Verify it's untracked
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.contains("?? src/"));

    // Commit using commit-files (stages then commits)
    let output = git_commit_files(dir, &["src", "--", "-m", "Add main.rs"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify commit
    let show = git(dir, &["show", "--name-only", "--format="]);
    assert!(show.contains("src/main.rs"));

    // Working tree should be clean
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.trim().is_empty(), "status should be clean: {status}");
}

#[test]
fn commits_modified_unstaged_file() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);
    git(dir, &["commit", "-m", "Add main.rs"]);

    // Modify without staging
    fs::write(dir.join("src/main.rs"), "fn main() { println!(\"hi\"); }\n").unwrap();

    // Verify it's modified but not staged
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.contains(" M src/main.rs"));

    // Commit using commit-files
    let output = git_commit_files(dir, &["src", "--", "-m", "Update main.rs"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the new content is committed
    let content = git(dir, &["show", "HEAD:src/main.rs"]);
    assert!(content.contains("println"));
}

#[test]
fn preserves_already_staged_changes_at_other_paths() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Stage a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.join("tests/test.rs"), "// test\n").unwrap();
    git(dir, &["add", "tests/test.rs"]);

    // commit-files on src (not tests)
    let output = git_commit_files(dir, &["src", "--", "-m", "Add main.rs"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // tests/test.rs should still be staged
    let status = git(dir, &["status", "--porcelain"]);
    assert!(
        status.contains("A  tests/test.rs"),
        "tests/test.rs should still be staged: {status}"
    );

    // Only src/main.rs should be in the commit
    let show = git(dir, &["show", "--name-only", "--format="]);
    assert!(show.contains("src/main.rs"));
    assert!(!show.contains("tests/test.rs"));
}

#[test]
fn dry_run_shows_what_would_be_committed() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create unstaged file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();

    let log_before = git(dir, &["rev-list", "--count", "HEAD"]);

    // Dry run
    let output = git_commit_files(dir, &["-n", "src", "--", "-m", "Dry run"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src/main.rs"));

    // Commit count unchanged
    let log_after = git(dir, &["rev-list", "--count", "HEAD"]);
    assert_eq!(log_before, log_after);
}

#[test]
fn amend_with_working_tree_changes() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);
    git(dir, &["commit", "-m", "Add main.rs"]);

    let log_before = git(dir, &["rev-list", "--count", "HEAD"]);

    // Modify without staging
    fs::write(dir.join("src/main.rs"), "fn main() { /* amended */ }\n").unwrap();

    // Amend using commit-files
    let output = git_commit_files(dir, &["src", "--", "--amend", "--no-edit"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Commit count unchanged (amend)
    let log_after = git(dir, &["rev-list", "--count", "HEAD"]);
    assert_eq!(log_before, log_after);

    // Verify amended content
    let content = git(dir, &["show", "HEAD:src/main.rs"]);
    assert!(content.contains("amended"));
}

#[test]
fn commits_already_staged_deletion() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/obsolete.rs"), "// to be deleted\n").unwrap();
    git(dir, &["add", "src/obsolete.rs"]);
    git(dir, &["commit", "-m", "Add obsolete file"]);

    // Delete and stage the deletion
    fs::remove_file(dir.join("src/obsolete.rs")).unwrap();
    git(dir, &["add", "src/obsolete.rs"]);

    // Verify it shows as staged deletion
    let status = git(dir, &["status", "--porcelain"]);
    assert!(
        status.contains("D  src/obsolete.rs"),
        "should show staged deletion: {status}"
    );

    // Commit using commit-files with exact file path (not directory)
    // This reproduces the bug: git add fails on already-staged deleted file
    let output = git_commit_files(dir, &["src/obsolete.rs", "--", "-m", "Remove obsolete file"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify file is gone from HEAD
    let show = git(dir, &["show", "--name-status", "--format="]);
    assert!(
        show.contains("D\tsrc/obsolete.rs"),
        "should show deletion in commit: {show}"
    );

    // Working tree should be clean
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.trim().is_empty(), "status should be clean: {status}");
}

#[test]
fn bails_when_staged_changes_differ_from_working_tree() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "v0\n").unwrap();
    git(dir, &["add", "src/main.rs"]);
    git(dir, &["commit", "-m", "Add main.rs"]);

    // Stage v1
    fs::write(dir.join("src/main.rs"), "v1\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Working tree has v2
    fs::write(dir.join("src/main.rs"), "v2\n").unwrap();

    // Verify status shows both staged and unstaged (MM)
    let status = git(dir, &["status", "--porcelain"]);
    assert!(status.contains("MM src/main.rs"), "expected MM status: {status}");

    // commit-files should bail
    let output = git_commit_files(dir, &["src", "--", "-m", "Should fail"]);
    assert!(
        !output.status.success(),
        "should have failed but succeeded"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("staged changes") && stderr.contains("differ from working tree"),
        "expected helpful error message: {stderr}"
    );
}
