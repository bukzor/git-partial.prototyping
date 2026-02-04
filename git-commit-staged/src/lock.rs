//! Git index lock management.
//!
//! Provides exclusive locking via `.git/index.lock` to serialize
//! with other git processes during commit operations.

use anyhow::{Context, Result};
use git2::Repository;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

/// RAII guard for `.git/index.lock`.
///
/// Acquires the lock on creation, releases on drop.
/// While held, no other git process can modify the index or HEAD.
pub struct IndexLock {
    path: PathBuf,
    #[allow(dead_code)]
    file: File,
}

impl IndexLock {
    /// Acquire `.git/index.lock` exclusively.
    ///
    /// Fails immediately if lock is held by another process (no retry/wait).
    ///
    /// # Errors
    /// Returns an error if the repository cannot be opened or the lock is already held.
    pub fn acquire() -> Result<Self> {
        let repo = Repository::open_from_env().context("failed to open repository")?;
        Self::acquire_for_repo(&repo)
    }

    /// Acquire lock for a specific repository.
    ///
    /// # Errors
    /// Returns an error if the lock file cannot be created (already held or permission denied).
    pub fn acquire_for_repo(repo: &Repository) -> Result<Self> {
        let lock_path = repo.path().join("index.lock");

        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .with_context(|| {
                format!(
                    "failed to acquire index lock at {}\n\
                     Another git process may be running. If not, remove the stale lock file.",
                    lock_path.display()
                )
            })?;

        Ok(Self {
            path: lock_path,
            file,
        })
    }

    /// Acquire lock for a repository at a specific path.
    ///
    /// # Errors
    /// Returns an error if the repository cannot be opened or the lock is already held.
    #[cfg(test)]
    pub fn acquire_at(path: &std::path::Path) -> Result<Self> {
        let repo = Repository::open(path).context("failed to open repository")?;
        Self::acquire_for_repo(&repo)
    }
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            eprintln!("warning: failed to remove index lock: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Create initial commit so we have a valid repo state
        let sig = repo.signature().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();

        dir
    }

    #[test]
    fn lock_creates_file() {
        let dir = setup_repo();

        let lock_path = dir.path().join(".git/index.lock");
        assert!(!lock_path.exists());

        {
            let _lock = IndexLock::acquire_at(dir.path()).unwrap();
            assert!(lock_path.exists());
        }

        assert!(!lock_path.exists());
    }

    #[test]
    fn double_lock_fails() {
        let dir = setup_repo();

        let _lock1 = IndexLock::acquire_at(dir.path()).unwrap();
        let result = IndexLock::acquire_at(dir.path());

        assert!(result.is_err());
    }
}
