//! CLI execution helpers shared between git-commit-staged and git-commit-files.

use anyhow::{bail, Context, Result};
use git2::{Index, IndexAddOption, Repository};
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

/// Result of staging paths to a temp index.
pub struct StageResult {
    /// Path to temp index with staged content
    pub temp_index_path: PathBuf,
    /// Path to real index (for rename)
    pub real_index_path: PathBuf,
    /// Entries that were staged (diff from HEAD)
    pub staged_entries: Vec<StagedEntry>,
}

/// Stage paths from working tree to a temp index.
///
/// Uses `update_all` for tracked files (handles modifications and deletions)
/// plus `add_all` for new untracked files.
///
/// Returns the temp index path and staged entries. Caller decides whether to:
/// - Rename temp → real (commit path)
/// - Delete temp (dry-run path)
///
/// # Errors
/// Returns an error if paths is empty or staging fails.
pub fn stage_paths_to_temp(paths: &[UnglobbedPath]) -> Result<StageResult> {
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

    // Write to temp path (avoids conflict with index.lock we hold)
    let git_dir = repo.path();
    let temp_path = git_dir.join(format!("index.stage.{}", std::process::id()));

    let mut temp_index =
        Index::open(&temp_path).context("failed to create temp index for staging")?;

    for entry in index.iter() {
        temp_index.add(&entry).with_context(|| {
            format!(
                "failed to copy entry {}",
                String::from_utf8_lossy(&entry.path)
            )
        })?;
    }

    temp_index.write().context("failed to write temp index")?;

    // Find what was staged (diff HEAD to temp index)
    let head_tree = repo
        .head()
        .context("failed to get HEAD")?
        .peel_to_tree()
        .context("failed to peel HEAD to tree")?;

    let staged_entries = find_staged_in_index(&repo, &temp_index, &head_tree, paths)?;

    Ok(StageResult {
        temp_index_path: temp_path,
        real_index_path: git_dir.join("index"),
        staged_entries,
    })
}

/// Find entries in an index that differ from HEAD at the given paths.
fn find_staged_in_index(
    repo: &Repository,
    index: &Index,
    head_tree: &git2::Tree,
    paths: &[UnglobbedPath],
) -> Result<Vec<StagedEntry>> {
    let matches = |p: &Path| paths.iter().any(|t| p.starts_with(t.as_ref()));

    let diff = repo
        .diff_tree_to_index(Some(head_tree), Some(index), None)
        .context("failed to diff HEAD to index")?;

    let mut staged = Vec::new();

    for delta in diff.deltas() {
        let path = delta.new_file().path().context("diff delta has no path")?;

        if !matches(path) {
            continue;
        }

        let path_str = path
            .to_str()
            .context("path is not valid UTF-8")?
            .to_owned();

        let entry = if delta.status() == git2::Delta::Deleted {
            (path_str, None)
        } else {
            let f = delta.new_file();
            (path_str, Some((f.id(), u32::from(f.mode()))))
        };

        staged.push(entry);
    }

    Ok(staged)
}

/// Commit the temp index by renaming it to the real index.
///
/// # Errors
/// Returns an error if the rename fails.
pub fn commit_staged_index(stage_result: &StageResult) -> Result<()> {
    std::fs::rename(&stage_result.temp_index_path, &stage_result.real_index_path)
        .context("failed to rename temp index to real index")
}

/// Discard the temp index (for dry-run).
///
/// # Errors
/// Returns an error if the temp index exists but cannot be removed.
pub fn discard_staged_index(stage_result: &StageResult) -> Result<()> {
    if stage_result.temp_index_path.exists() {
        std::fs::remove_file(&stage_result.temp_index_path)
            .context("failed to remove temp index")?;
    }
    Ok(())
}

