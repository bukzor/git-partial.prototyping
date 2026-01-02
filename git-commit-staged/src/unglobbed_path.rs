//! Pathspec handling with directory expansion.

use std::ffi::CString;
use std::path::{Path, PathBuf};

use git2::{Error, IntoCString};

/// A path that has been "unglobbed" - directories expanded to their file contents.
///
/// This represents a concrete file path (existing, deleted, or symlink) but never
/// a directory. The invariant is enforced by [`from_paths`], which recursively
/// expands any directories.
#[derive(Debug, Clone)]
pub struct UnglobbedPath(PathBuf);

impl UnglobbedPath {
    /// Expand paths: directories become their recursive file contents.
    /// Non-directory paths pass through (including deleted files).
    #[must_use]
    pub fn from_paths(paths: &[PathBuf]) -> Vec<Self> {
        let mut result = Vec::new();
        for path in paths {
            if path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(path) {
                    let children: Vec<PathBuf> =
                        entries.filter_map(Result::ok).map(|e| e.path()).collect();
                    result.extend(Self::from_paths(&children));
                }
            } else {
                result.push(Self(path.clone()));
            }
        }
        result
    }

}

impl AsRef<Path> for UnglobbedPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl IntoCString for UnglobbedPath {
    fn into_c_string(self) -> Result<CString, Error> {
        self.0.into_c_string()
    }
}
