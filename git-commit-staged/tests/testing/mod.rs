//! Shared helpers for integration tests.
//!
//! Each `tests/*.rs` is a separate test crate; shared code lives in
//! `tests/testing/mod.rs` so Cargo treats it as a module of every test
//! crate that imports it (`mod testing;`) rather than auto-discovering
//! it as its own test binary. Each test crate uses a different subset
//! of the helpers below; the module-level `dead_code` allow keeps
//! unused items in any one crate from warning.

#![allow(dead_code)]

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

/// Run a `git` command in `dir`; panic on non-zero exit.
pub fn git(dir: &Path, args: &[&str]) -> String {
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

/// `git init` with `init.templateDir` overridden to empty. This keeps
/// tests hermetic against developer-local `init.templateDir` hooks —
/// notably git-localhost-store's auto-migration hooks, which would
/// otherwise relocate every test repo's `.git` into the user's central
/// `~/.local/state/git-localhost-store/repos/` and leave an orphaned
/// entry behind after the temp dir is cleaned up.
pub fn git_init(dir: &Path) {
    git(dir, &["-c", "init.templateDir=", "init", "-b", "main"]);
    git(dir, &["config", "user.email", "test@test.com"]);
    git(dir, &["config", "user.name", "Test User"]);
}

/// Create a temp repo with an initial commit.
pub fn setup_repo() -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let dir = tmp.path();

    git_init(dir);
    fs::write(dir.join("README.md"), "# Test Repo\n").unwrap();
    git(dir, &["add", "README.md"]);
    git(dir, &["commit", "-m", "Initial commit"]);

    tmp
}

/// Create a temp repo without any commits.
pub fn setup_empty_repo() -> TempDir {
    let tmp = TempDir::new().expect("failed to create temp dir");
    git_init(tmp.path());
    tmp
}
