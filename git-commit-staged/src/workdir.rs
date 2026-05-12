//! Resolve repository paths logically, preserving symlinks.
//!
//! # Why not `git2::Repository::workdir`?
//!
//! libgit2's `git_repository_workdir` infers the working directory as
//! `parent(canonical(gitdir))` for non-bare repos with no `core.worktree`
//! set. That breaks when `.git` is a symlink whose target is an absolute
//! path outside the working directory.
//!
//! The triggering layout is git-localhost-store
//! (`~/.local/share/git-localhost-store/`), where every workdir's `.git`
//! is a symlink into a path-encoded central store:
//!
//! ```text
//! <workdir>/.git -> ~/.local/state/git-localhost-store/repos/<encoded>/
//! ```
//!
//! libgit2 follows the symlink, canonicalizes, and returns
//! `~/.local/state/git-localhost-store/repos` as the workdir. Wrong.
//!
//! Setting `core.worktree` per gitdir would silence libgit2 but couple
//! each gitdir to a fixed workdir path, defeating the layout's whole
//! point: the workdir can move freely, only the symlink updates. (The
//! ADR at `~/.local/share/git-localhost-store/docs/adr/`
//! `2026-04-30-000-switch-from-gitfile-to-symlink-layout.md` records why
//! that layout was chosen.)
//!
//! # Why not `std::fs::canonicalize`?
//!
//! Canonicalization resolves symlinks against the filesystem's *current*
//! state. The result captures whatever the link points at right now;
//! if the link target later moves, the captured path is stale.
//!
//! The logical, symlinked form — what `$PWD` holds and what
//! `git rev-parse --show-toplevel` reports — stays valid as long as the
//! link itself does. We prefer it for any path that may be persisted,
//! compared across calls, or shown to the user. Walking up from such a
//! path with textual `parent()` reaches the workdir without ever
//! touching symlink resolution.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Shell-style logical working directory, preserving symlinks.
///
/// Reads `$PWD`. POSIX shells maintain it as an absolute path across
/// `cd`; if it's missing or non-absolute, the tool was launched from
/// an environment that violates our contract — panic so the caller can
/// fix it, rather than silently substituting `std::env::current_dir()`
/// (which resolves symlinks on Linux and produces a different path
/// than what the user sees as `$PWD`).
///
/// CLI entry points should call [`ensure_pwd`] at the top of `main` to
/// repair `$PWD` if a launcher like `git -C foo` left it stale.
///
/// # Panics
/// Panics if `$PWD` is unset or not absolute.
pub fn logical_cwd() -> PathBuf {
    let pwd = std::env::var_os("PWD").expect("$PWD must be set");
    let pwd = PathBuf::from(pwd);
    assert!(pwd.is_absolute(), "$PWD must be absolute: {}", pwd.display());
    pwd
}

/// Ensure `$PWD` names the current working directory.
///
/// POSIX shells maintain `$PWD` across `cd` so it reflects the
/// logical (possibly symlinked) path the user navigated through.
/// Some launchers — notably `git -C foo` exec'ing us directly without
/// going through `/bin/sh` — `chdir()` us but leave `$PWD` pointing at
/// the parent shell's cwd. (Shell-script subcommands appear to "fix"
/// this only because `/bin/sh` resets `$PWD = getcwd()` on startup;
/// binaries inherit the stale value as-is.)
///
/// Repair `$PWD` when stale, leave it alone otherwise:
/// - "Stale" = `$PWD` and `.` resolve to different inodes (or `$PWD`
///   is unset/non-absolute/unreadable).
/// - When they match, preserve whatever logical/symlinked form `$PWD`
///   already holds — a shell's `cd through-a-symlink` set it up that
///   way deliberately and we shouldn't clobber it with the canonical
///   form `getcwd()` returns.
///
/// When `$PWD` *is* stale, we have no choice but to substitute
/// `getcwd()`'s canonical form: git already consumed its `-C` argument
/// before `exec`'ing us, so the original logical path is unrecoverable.
/// This trades the user's symlinked form for a consistent
/// `$PWD`-matches-cwd invariant; the alternative (leave `$PWD`
/// pointing somewhere else) silently breaks anything downstream that
/// reads it.
///
/// Call once, at the top of `main`, before any other code reads
/// `$PWD` or spawns subprocesses.
///
/// # Panics
/// Panics if a repair is needed and `getcwd()` fails.
pub fn ensure_pwd() {
    if pwd_names_cwd() {
        return;
    }
    let cwd = std::env::current_dir().expect("failed to read current directory");
    // SAFETY: documented as a top-of-main call, before any threads
    // spawn, so no concurrent env access can race with this write.
    unsafe { std::env::set_var("PWD", &cwd) };
}

/// True iff `$PWD` is absolute and names the same inode as `.`.
fn pwd_names_cwd() -> bool {
    let Some(pwd) = std::env::var_os("PWD") else { return false };
    let pwd = PathBuf::from(pwd);
    pwd.is_absolute() && samefile(Path::new("."), &pwd)
}

/// True iff `a` and `b` name the same inode (Python's `os.path.samefile`).
///
/// Returns `false` if either path can't be `stat`'d — `samefile`'s
/// "they aren't the same file" is the right answer for any reason a
/// caller would care, including "I can't see one of them."
///
/// Rust's stdlib has no equivalent; the `same-file` crate exists but
/// dragging in a dependency for one `stat` comparison isn't worth it.
pub(crate) fn samefile(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let (Ok(a), Ok(b)) = (std::fs::metadata(a), std::fs::metadata(b)) else {
        return false;
    };
    a.dev() == b.dev() && a.ino() == b.ino()
}

/// Walk up from `start` to the first directory containing a `.git` entry.
///
/// Accepts any `.git` shape: directory, symlink (resolved or broken), or
/// gitfile. `start` must be an absolute, textually-normalized path
/// inside the working tree — typically [`logical_cwd`] or a
/// `gix_path::normalize` of a user-supplied directory against it. Do
/// **not** pass a canonicalized path; see the module docs.
///
/// # Errors
/// Returns an error if no `.git` entry is found at or above `start`.
pub fn repo_workdir(start: &Path) -> Result<PathBuf> {
    let mut cur = start;
    loop {
        // `symlink_metadata` doesn't follow the link, so it reports any
        // `.git` entry — including broken symlinks, which we want to
        // surface rather than skip.
        if std::fs::symlink_metadata(cur.join(".git")).is_ok() {
            return Ok(cur.to_path_buf());
        }
        cur = cur
            .parent()
            .with_context(|| format!("no .git found at or above {}", start.display()))?;
    }
}
