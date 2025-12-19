//! Temporary index file creation for path-scoped commits.

use anyhow::{Context, Result};
use git2::{Index, Repository};
use std::path::{Path, PathBuf};

use crate::StagedEntry;

/// Write a temporary index file containing HEAD + specified staged entries.
///
/// The temp index is placed in `.git/index.commit-staged.<pid>` following the
/// pattern from `path-scoped-index-commit-workflow.md`.
///
/// # Errors
/// Returns an error if HEAD cannot be resolved, index file cannot be created,
/// or index operations fail.
pub fn write_temp_index(repo: &Repository, entries: &[StagedEntry]) -> Result<PathBuf> {
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
