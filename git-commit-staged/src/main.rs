use anyhow::{Context, Result, bail};
use clap::Parser;
use git2::{Index, Oid, Repository};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "git-commit-staged")]
#[command(about = "Commit staged changes at specific paths only")]
#[command(
    long_about = "Unlike `git commit -- paths`, this commits from the index, not the working copy."
)]
struct Args {
    /// Paths to commit (only staged changes at these paths)
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Commit message
    #[arg(short, long, required = true)]
    message: String,

    /// Run as if git was started in <path>
    #[arg(short = 'C', long = "directory", default_value = ".")]
    directory: PathBuf,

    /// Show what would be committed without committing
    #[arg(short = 'n', long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Canonicalize scope root (the -C directory) - this resolves symlinks
    let scope_root = std::fs::canonicalize(&args.directory)
        .with_context(|| format!("failed to canonicalize {}", args.directory.display()))?;

    let repo = Repository::discover(&scope_root)
        .with_context(|| format!("failed to discover repository at {}", scope_root.display()))?;

    let repo_root = repo
        .workdir()
        .context("repository has no workdir (bare repo?)")?;
    let repo_root =
        std::fs::canonicalize(repo_root).context("failed to canonicalize repo workdir")?;

    // Resolve user paths to repo-relative paths
    let resolved_paths = resolve_paths(&args.paths, &scope_root, &repo_root)?;

    let staged_entries = find_staged_entries(&repo, &resolved_paths)?;

    if staged_entries.is_empty() {
        bail!("no staged changes at specified paths");
    }

    println!("Files to commit:");
    for (path, data) in &staged_entries {
        let status = if data.is_some() { "M" } else { "D" };
        println!("  {status} {path}");
    }
    println!();

    if args.dry_run {
        println!("Dry run - not committing");
        println!();
        println!("Would commit with message:");
        println!("  {}", args.message);
        return Ok(());
    }

    let commit_oid = create_commit(&repo, &staged_entries, &args.message)?;

    // Remove committed paths from main index (reset to new HEAD)
    unstage_paths(&repo, &staged_entries)?;

    println!("Created commit: {commit_oid}");
    print_commit_oneline(&repo, commit_oid)?;

    Ok(())
}

/// Entry: (path, `blob_oid`, filemode) - None means deletion
type StagedEntry = (String, Option<(Oid, u32)>);

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
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .context("diff delta has no path")?;

        if !path_matches(path, paths) {
            continue;
        }

        let path_str = path.to_str().context("path is not valid UTF-8")?.to_owned();

        let entry = match delta.status() {
            git2::Delta::Deleted => (path_str, None),
            _ => {
                let f = delta.new_file();
                (path_str, Some((f.id(), u32::from(f.mode()))))
            }
        };

        staged.push(entry);
    }

    Ok(staged)
}

/// Create a commit with only the specified entries changed from HEAD
fn create_commit(repo: &Repository, entries: &[StagedEntry], message: &str) -> Result<Oid> {
    let head = repo.head().context("failed to get HEAD")?;
    let head_commit = head.peel_to_commit().context("failed to get HEAD commit")?;
    let head_tree = head_commit.tree().context("failed to get HEAD tree")?;

    // Build a new index with HEAD as base
    let mut index = Index::new().context("failed to create index")?;
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

    // Write tree from our custom index
    let tree_oid = index.write_tree_to(repo).context("failed to write tree")?;
    let tree = repo
        .find_tree(tree_oid)
        .context("failed to find written tree")?;

    // Create commit
    let sig = repo
        .signature()
        .context("failed to get default signature")?;

    let commit_oid = repo
        .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&head_commit])
        .context("failed to create commit")?;

    Ok(commit_oid)
}

/// Remove the committed paths from the main index (they now match HEAD)
fn unstage_paths(repo: &Repository, entries: &[StagedEntry]) -> Result<()> {
    let mut index = repo.index().context("failed to read index")?;
    let head = repo.head().context("failed to get HEAD")?;
    let head_tree = head.peel_to_tree().context("failed to get HEAD tree")?;

    for (path, data) in entries {
        match data {
            Some(_) => {
                // Addition or modification: reset index to match new HEAD
                let tree_entry = head_tree
                    .get_path(Path::new(path))
                    .with_context(|| format!("committed file not found in new HEAD: {path}"))?;

                let mode = u32::try_from(tree_entry.filemode()).with_context(|| {
                    format!("invalid filemode for {path}: {}", tree_entry.filemode())
                })?;

                let entry = git2::IndexEntry {
                    ctime: git2::IndexTime::new(0, 0),
                    mtime: git2::IndexTime::new(0, 0),
                    dev: 0,
                    ino: 0,
                    mode,
                    uid: 0,
                    gid: 0,
                    file_size: 0,
                    id: tree_entry.id(),
                    flags: 0,
                    flags_extended: 0,
                    path: path.as_bytes().to_vec(),
                };
                index
                    .add(&entry)
                    .with_context(|| format!("failed to reset index entry: {path}"))?;
            }
            None => {
                // Deletion: already removed from index by git rm, nothing to do
            }
        }
    }

    index.write().context("failed to write index")?;
    Ok(())
}

fn print_commit_oneline(repo: &Repository, oid: Oid) -> Result<()> {
    let commit = repo.find_commit(oid).context("failed to find commit")?;
    let short_id = commit
        .as_object()
        .short_id()
        .context("failed to get short id")?;
    let short_id_str = short_id.as_str().context("short id is not valid UTF-8")?;
    let summary = commit
        .summary()
        .context("commit message is not valid UTF-8")?;
    println!("{short_id_str} {summary}");
    Ok(())
}

/// Check if an entry path matches any of the requested paths (exact or descendant)
fn path_matches(entry_path: &Path, requested_paths: &[PathBuf]) -> bool {
    requested_paths
        .iter()
        .any(|p| entry_path == p || entry_path.starts_with(p))
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
