//! CLI execution helpers shared between git-commit-staged and git-commit-files.

use anyhow::{bail, Context, Result};
use git2::{IndexAddOption, Repository};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::unglobbed_path::UnglobbedPath;
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

/// Check that no staged changes would be destroyed by commit-files.
///
/// Bails if index differs from BOTH HEAD and working tree at any path.
/// If index matches HEAD (nothing staged) or matches working tree
/// (already staged what we're committing), we're fine.
///
/// # Errors
/// Returns an error if staged changes would be lost, or if git operations fail.
pub fn check_no_staged_changes(paths: &[UnglobbedPath]) -> Result<()> {
    use std::collections::HashSet;

    let repo = Repository::open_from_env().context("failed to open repository")?;
    let index = repo.index().context("failed to get index")?;
    let head = repo.head()?.peel_to_tree()?;

    let matches = |p: &Path| paths.iter().any(|t| p.starts_with(t.as_ref()));

    let staged: HashSet<PathBuf> = repo
        .diff_tree_to_index(Some(&head), Some(&index), None)?
        .deltas()
        .filter_map(|d| d.new_file().path())
        .filter(|p| matches(p))
        .map(Path::to_path_buf)
        .collect();

    let unstaged: HashSet<PathBuf> = repo
        .diff_index_to_workdir(Some(&index), None)?
        .deltas()
        .filter_map(|d| d.new_file().path())
        .filter(|p| matches(p))
        .map(Path::to_path_buf)
        .collect();

    let conflicts: Vec<_> = staged.intersection(&unstaged).collect();
    if !conflicts.is_empty() {
        let list = conflicts.iter().map(|p| format!("  {}", p.display())).collect::<Vec<_>>().join("\n");
        bail!(
            "staged changes at these paths differ from working tree:\n{list}\n\n\
             These would be overwritten. Either:\n  \
               git commit-staged <paths>   # commit staged changes first\n  \
               git reset <paths>           # discard staged changes\n\
             Then retry."
        );
    }

    Ok(())
}

/// Stage paths from working tree using git2.
///
/// Uses `update_all` for tracked files (handles modifications and deletions)
/// plus `add_all` for new untracked files.
///
/// # Errors
/// Returns an error if paths is empty or staging fails.
pub fn stage_paths(paths: &[UnglobbedPath]) -> Result<()> {
    if paths.is_empty() {
        bail!("no paths specified");
    }

    let repo = Repository::open_from_env().context("failed to open repository")?;
    let mut index = repo.index().context("failed to get index")?;

    // update_all: sync index with working tree for tracked files (modifications + deletions)
    index
        .update_all(paths, None)
        .context("failed to update index from working tree")?;

    // add_all: also stage new untracked files
    index
        .add_all(paths, IndexAddOption::DEFAULT, None)
        .context("failed to add paths to index")?;

    index.write().context("failed to write index")?;

    Ok(())
}
