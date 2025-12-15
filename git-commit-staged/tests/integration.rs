//! Integration tests for git-commit-staged
//!
//! These tests create real git repositories in temp directories.

use std::fs;
use std::path::Path;
use std::process::Command;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
fn commits_staged_file_at_specified_path() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and stage a file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Commit it
    let output = git_commit_staged(dir, &["src", "-m", "Add main.rs"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
    let output = git_commit_staged(dir, &["src", "-m", "Add v1"]);
    assert!(output.status.success());

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
    let output = git_commit_staged(dir, &["src", "-m", "Add main.rs only"]);
    assert!(output.status.success());

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
    let output = git_commit_staged(dir, &["../outside", "-m", "Should fail"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("escapes scope"));
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

#[test]
fn commits_staged_mode_change() {
    let tmp = setup_repo();
    let dir = tmp.path();

    // Create and commit a non-executable file
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/script.sh"), "#!/bin/bash\necho hello\n").unwrap();
    git(dir, &["add", "src/script.sh"]);
    git(dir, &["commit", "-m", "Add script"]);

    // chmod +x and stage
    std::process::Command::new("chmod")
        .args(["+x", "src/script.sh"])
        .current_dir(dir)
        .status()
        .unwrap();
    git(dir, &["add", "src/script.sh"]);

    let status = git(dir, &["status", "--porcelain"]);
    assert_eq!(status, "M  src/script.sh\n");

    // Commit the mode change
    let output = git_commit_staged(dir, &["src", "-m", "Make script executable"]);
    assert!(
        output.status.success(),
        "should commit mode change, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify the commit happened
    let log = git(dir, &["log", "--oneline", "-1"]);
    assert!(log.contains("Make script executable"));

    // Verify the executable bit was actually committed
    // Check out the file and verify it's executable
    git(dir, &["checkout", "HEAD", "--", "src/script.sh"]);
    let metadata = std::fs::metadata(dir.join("src/script.sh")).unwrap();
    let mode = metadata.permissions().mode();
    assert_eq!(
        mode & 0o111,
        0o111,
        "file should be executable, mode: {:o}",
        mode
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
    assert!(status.contains("T  src/link.txt"), "should show typechange: {status}");

    // Commit the typechange
    let output = git_commit_staged(dir, &["src", "-m", "Change to symlink"]);
    assert!(
        output.status.success(),
        "should commit typechange, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
    let output = git_commit_staged(dir, &["src", "-m", "Remove obsolete file"]);
    assert!(
        output.status.success(),
        "should commit deletion, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

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
    let output = git_commit_staged(dir, &["src", "-m", "Rename file"]);
    assert!(
        output.status.success(),
        "should commit rename, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify both old gone and new exists
    let show = git(dir, &["show", "--name-status", "--format="]);
    assert!(show.contains("src/new_name.rs"), "new file should exist: {show}");
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
    let output = git_commit_staged(dir, &["src", "-m", "Rename file"]);
    assert!(
        output.status.success(),
        "should commit rename with detection enabled, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify both old gone and new exists
    let show = git(dir, &["show", "--name-status", "--format="]);
    assert!(show.contains("src/new_name.rs"), "new file should exist: {show}");
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

#[test]
fn errors_on_empty_repo() {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let dir = tmp.path();

    git(dir, &["init"]);
    git(dir, &["config", "user.email", "test@test.com"]);
    git(dir, &["config", "user.name", "Test User"]);

    // Create and stage a file but don't commit
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    git(dir, &["add", "src/main.rs"]);

    // Tool should error - no HEAD exists
    let output = git_commit_staged(dir, &["src", "-m", "First commit"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("HEAD") || stderr.contains("head"),
        "error should mention HEAD: {stderr}"
    );
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
    let output = git_commit_staged(dir, &["src", "-m", "Nothing staged"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no staged changes"));
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
    let output = git_commit_staged(dir, &["src", "-m", "Add main.rs"]);
    assert!(output.status.success());

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
