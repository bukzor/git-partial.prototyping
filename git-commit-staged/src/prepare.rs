//! Prepare staged changes at specific paths for commit.

use anyhow::{bail, Context, Result};
use git2::Repository;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::index::write_temp_index;
use crate::{PrepareResult, StagedEntry};

/// Prepare staged changes at specific paths for commit.
///
/// Unlike `git commit -- paths`, this commits from the index, not the working copy.
///
/// Creates a temporary index file containing HEAD tree + staged changes at the
/// specified paths. The caller should then exec `git commit` with `GIT_INDEX_FILE`
/// set to the returned temp path.
///
/// # Arguments
/// * `paths` - Paths to commit (only staged changes at these paths)
/// * `directory` - Run as if git was started in this directory
/// * `dry_run` - If true, show what would be committed without creating temp index
///
/// # Errors
/// Returns an error if:
/// - The directory cannot be canonicalized
/// - No git repository is found
/// - The repository has no workdir (bare repo)
/// - A path escapes the scope directory
/// - No HEAD commit exists (empty repo)
/// - No staged changes exist at the specified paths
/// - Git operations fail (index read, tree write)
pub fn prepare_staged_commit(
    paths: &[PathBuf],
    directory: &Path,
    dry_run: bool,
) -> Result<PrepareResult> {
    // Canonicalize scope root (the -C directory) - this resolves symlinks
    let scope_root = std::fs::canonicalize(directory)
        .with_context(|| format!("failed to canonicalize {}", directory.display()))?;

    let repo = Repository::discover(&scope_root)
        .with_context(|| format!("failed to discover repository at {}", scope_root.display()))?;

    let repo_root = repo
        .workdir()
        .context("repository has no workdir (bare repo?)")?;
    let repo_root =
        std::fs::canonicalize(repo_root).context("failed to canonicalize repo workdir")?;

    // Resolve user paths to repo-relative paths
    let resolved_paths = resolve_paths(paths, &scope_root, &repo_root)?;

    let staged_entries = find_staged_entries(&repo, &resolved_paths)?;

    if staged_entries.is_empty() {
        bail!("no staged changes at specified paths");
    }

    if dry_run {
        return Ok(PrepareResult {
            staged_entries,
            temp_index_path: None,
        });
    }

    let temp_index_path = write_temp_index(&repo, &staged_entries)?;

    Ok(PrepareResult {
        staged_entries,
        temp_index_path: Some(temp_index_path),
    })
}

/// Resolve user-provided paths to repo-relative paths.
///
/// - Paths are resolved relative to `scope_root` (the -C directory)
/// - Paths must stay within `scope_root` (no escaping via ..)
/// - Returns paths relative to `repo_root` for index comparison
fn resolve_paths(
    user_paths: &[PathBuf],
    scope_root: &Path,
    repo_root: &Path,
) -> Result<Vec<PathBuf>> {
    user_paths
        .iter()
        .map(|user_path| {
            // Make absolute by joining with scope_root
            let absolute = scope_root.join(user_path);

            // Normalize (handles . and ..)
            let normalized = gix_path::normalize(Cow::Owned(absolute), scope_root)
                .context("path normalization failed")?;

            // Check path stays within scope
            if !normalized.starts_with(scope_root) {
                bail!(
                    "{} escapes scope {}",
                    user_path.display(),
                    scope_root.display()
                );
            }

            // Strip repo_root to get repo-relative path
            normalized
                .strip_prefix(repo_root)
                .map(Path::to_path_buf)
                .with_context(|| {
                    format!(
                        "{} is outside repository {}",
                        user_path.display(),
                        repo_root.display()
                    )
                })
        })
        .collect()
}

/// Find entries in the index that differ from HEAD at the given paths
fn find_staged_entries(repo: &Repository, paths: &[PathBuf]) -> Result<Vec<StagedEntry>> {
    let index = repo.index().context("failed to read index")?;
    let head_tree = repo
        .head()
        .context("failed to get HEAD")?
        .peel_to_tree()
        .context("failed to peel HEAD to tree")?;

    let diff = repo
        .diff_tree_to_index(Some(&head_tree), Some(&index), None)
        .context("failed to diff HEAD to index")?;

    let mut staged = Vec::new();

    for delta in diff.deltas() {
        // Note: new_file().path() returns the path for all delta types from
        // diff_tree_to_index, including deletions. The old_file fallback was
        // dead code - git2 always populates new_file.path() for this diff type.
        let path = delta.new_file().path().context("diff delta has no path")?;

        if !path_matches(path, paths) {
            continue;
        }

        let path_str = path.to_str().context("path is not valid UTF-8")?.to_owned();

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

/// Check if an entry path matches any of the requested paths (exact or descendant)
pub(crate) fn path_matches(entry_path: &Path, requested_paths: &[PathBuf]) -> bool {
    requested_paths.iter().any(|p| entry_path.starts_with(p))
}
