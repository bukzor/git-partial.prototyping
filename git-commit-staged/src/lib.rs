//! git-commit-staged: Commit staged changes at specific paths only
//!
//! Unlike `git commit -- paths`, this commits from the index, not the working copy.
//!
//! # Architecture
//!
//! This library prepares a temporary index file containing HEAD + staged changes
//! at specified paths. The caller (CLI) then execs `git commit` with
//! `GIT_INDEX_FILE` set to this temp file, enabling full `git commit` features
//! (--amend, --fixup, -C, GPG signing, hooks, editor, etc.).
//!
//! # Modules
//!
//! - [`prepare`] - Core logic for preparing staged commits
//! - [`index`] - Temporary index file creation
//! - [`exec`] - CLI execution helpers

use anyhow::{bail, Context, Result};
use git2::Oid;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod exec;
pub mod index;
pub mod unglobbed_path;
pub mod prepare;

#[cfg(test)]
mod tests;

// Re-export main entry points
pub use prepare::prepare_staged_commit;

/// Result of preparing staged changes for commit
#[derive(Debug)]
pub struct PrepareResult {
    /// The entries that will be committed
    pub staged_entries: Vec<StagedEntry>,
    /// Path to the temporary index file (None for dry run)
    pub temp_index_path: Option<PathBuf>,
}

/// Entry: (path, `blob_oid`, filemode) - None means deletion
pub type StagedEntry = (String, Option<(Oid, u32)>);

/// Result of a commit operation
#[derive(Debug)]
pub struct CommitResult {
    /// The entries that were committed
    pub staged_entries: Vec<StagedEntry>,
}

/// Commit staged changes at specific paths only.
///
/// Unlike `git commit -- paths`, this commits from the index, not the working copy.
///
/// This is a convenience function that prepares a temp index and runs `git commit`.
/// For more control (e.g., passing additional git commit args), use
/// [`prepare_staged_commit`] and exec `git commit` yourself.
///
/// # Arguments
/// * `paths` - Paths to commit (only staged changes at these paths)
/// * `message` - Commit message
/// * `directory` - Run as if git was started in this directory
/// * `dry_run` - If true, show what would be committed without committing
///
/// # Errors
/// Returns an error if preparation fails or if `git commit` fails.
///
/// # Panics
/// Panics if `dry_run` is false but no temp index path is returned (internal invariant).
pub fn git_commit_staged(
    paths: &[PathBuf],
    message: &str,
    directory: &Path,
    dry_run: bool,
) -> Result<CommitResult> {
    let result = prepare_staged_commit(paths, directory, dry_run)?;

    if dry_run {
        return Ok(CommitResult {
            staged_entries: result.staged_entries,
        });
    }

    let temp_index_path = result
        .temp_index_path
        .expect("non-dry-run should have temp index");

    // Run git commit with the temp index
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .env("GIT_INDEX_FILE", &temp_index_path)
        .output()
        .context("failed to run git commit")?;

    // Always clean up the temp index
    let _ = std::fs::remove_file(&temp_index_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git commit failed: {stderr}");
    }

    Ok(CommitResult {
        staged_entries: result.staged_entries,
    })
}
