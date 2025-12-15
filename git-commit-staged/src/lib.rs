//! git-commit-staged: Commit staged changes at specific paths only
//!
//! Unlike `git commit -- paths`, this commits from the index, not the working copy.
//!
//! Architecture: This library prepares a temporary index file containing HEAD + staged
//! changes at specified paths. The caller (CLI) then execs `git commit` with
//! `GIT_INDEX_FILE` set to this temp file, enabling full `git commit` features
//! (--amend, --fixup, -C, GPG signing, hooks, editor, etc.).

use anyhow::{Context, Result, bail};
use git2::{Index, Oid, Repository};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::process::Command;

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
/// `prepare_staged_commit` and exec `git commit` yourself.
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

/// Write a temporary index file containing HEAD + specified staged entries.
///
/// The temp index is placed in `.git/index.commit-staged.<pid>` following the
/// pattern from `path-scoped-index-commit-workflow.md`.
fn write_temp_index(repo: &Repository, entries: &[StagedEntry]) -> Result<PathBuf> {
    let head = repo.head().context("failed to get HEAD")?;
    let head_commit = head.peel_to_commit().context("failed to get HEAD commit")?;
    let head_tree = head_commit.tree().context("failed to get HEAD tree")?;

    // Determine temp index path: .git/index.commit-staged.<pid>
    let git_dir = repo.path(); // .git directory
    let pid = std::process::id();
    let temp_index_path = git_dir.join(format!("index.commit-staged.{pid}"));

    // Create the temp index file and open it
    // Index::open creates a new file if it doesn't exist
    let mut index = Index::open(&temp_index_path).context("failed to create temp index file")?;

    // Read HEAD tree as the base
    index
        .read_tree(&head_tree)
        .context("failed to read HEAD tree into index")?;

    // Apply our staged entries
    for (path, data) in entries {
        match data {
            Some((oid, mode)) => {
                let entry = git2::IndexEntry {
                    ctime: git2::IndexTime::new(0, 0),
                    mtime: git2::IndexTime::new(0, 0),
                    dev: 0,
                    ino: 0,
                    mode: *mode,
                    uid: 0,
                    gid: 0,
                    file_size: 0,
                    id: *oid,
                    flags: 0,
                    flags_extended: 0,
                    path: path.as_bytes().to_vec(),
                };
                index
                    .add(&entry)
                    .with_context(|| format!("failed to add {path} to index"))?;
            }
            None => {
                index
                    .remove(Path::new(path), 0)
                    .with_context(|| format!("failed to remove {path} from index"))?;
            }
        }
    }

    // Write the index to disk
    index.write().context("failed to write temp index")?;

    Ok(temp_index_path)
}

/// Check if an entry path matches any of the requested paths (exact or descendant)
fn path_matches(entry_path: &Path, requested_paths: &[PathBuf]) -> bool {
    requested_paths.iter().any(|p| entry_path.starts_with(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod path_matching {
        use super::*;

        #[test]
        fn exact_file_match() {
            let entry = Path::new("src/main.rs");
            let requested = vec![PathBuf::from("src/main.rs")];
            assert!(path_matches(entry, &requested));
        }

        #[test]
        fn directory_contains_file() {
            let entry = Path::new("src/main.rs");
            let requested = vec![PathBuf::from("src")];
            assert!(path_matches(entry, &requested));
        }

        #[test]
        fn nested_directory_contains_file() {
            let entry = Path::new("src/foo/bar/baz.rs");
            let requested = vec![PathBuf::from("src/foo")];
            assert!(path_matches(entry, &requested));
        }

        #[test]
        fn no_match_different_file() {
            let entry = Path::new("src/main.rs");
            let requested = vec![PathBuf::from("src/lib.rs")];
            assert!(!path_matches(entry, &requested));
        }

        #[test]
        fn no_match_sibling_directory() {
            let entry = Path::new("src/main.rs");
            let requested = vec![PathBuf::from("tests")];
            assert!(!path_matches(entry, &requested));
        }

        #[test]
        fn no_false_prefix_match() {
            // "src/foo" should NOT match "src/foobar/baz.rs"
            let entry = Path::new("src/foobar/baz.rs");
            let requested = vec![PathBuf::from("src/foo")];
            assert!(!path_matches(entry, &requested));
        }

        #[test]
        fn multiple_requested_paths() {
            let entry = Path::new("tests/test_foo.rs");
            let requested = vec![PathBuf::from("src"), PathBuf::from("tests")];
            assert!(path_matches(entry, &requested));
        }

        #[test]
        fn root_matches_everything() {
            let entry = Path::new("src/deeply/nested/file.rs");
            let requested = vec![PathBuf::from("")];
            assert!(path_matches(entry, &requested));
        }
    }
}
