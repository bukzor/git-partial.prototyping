//! CLI execution helpers shared between git-commit-staged and git-commit-files.

use anyhow::{bail, Context, Result};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::StagedEntry;

/// Print dry-run output showing files that would be committed.
pub fn print_dry_run(staged_entries: &[StagedEntry]) {
    println!("Files to commit:");
    for (path, data) in staged_entries {
        let status = if data.is_some() { "M" } else { "D" };
        println!("  {status} {path}");
    }
}

/// Execute git commit with a temporary index file.
///
/// This function does not return on success - it replaces the current process
/// with `git commit`. On error, it cleans up the temp index and returns the error.
///
/// # Errors
/// Returns an error if `exec()` fails (e.g., git not found).
pub fn exec_git_commit(temp_index_path: &Path, passthrough_args: &[String]) -> Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("commit");
    cmd.args(passthrough_args);
    cmd.env("GIT_INDEX_FILE", temp_index_path);

    // exec() replaces this process with git commit
    let err = cmd.exec();

    // exec() only returns on error
    let _ = std::fs::remove_file(temp_index_path);
    Err(err).context("failed to exec git commit")
}

/// Stage paths from working tree using git add.
///
/// # Errors
/// Returns an error if paths is empty, git add fails to run, or git add exits non-zero.
pub fn stage_paths(paths: &[PathBuf]) -> Result<()> {
    if paths.is_empty() {
        bail!("no paths specified");
    }

    let mut cmd = Command::new("git");
    cmd.arg("add");
    cmd.arg("--");
    for path in paths {
        cmd.arg(path);
    }

    let output = cmd.output().context("failed to run git add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git add failed: {stderr}");
    }

    Ok(())
}
