---
constraints:
  - ../discovered-constraints.kb/libgit2-workdir-misidentifies-symlinked-gitdir.md
---

# Logical $PWD Path Resolution

For locating the working tree and resolving user-supplied directories, this
crate uses `$PWD` (logical, possibly symlinked) as the source of truth — not
libgit2's `repo.workdir()`, not `std::env::current_dir()`, and not
`std::fs::canonicalize`.

## Components

`src/workdir.rs`:

- **`logical_cwd()`**: reads `$PWD`. Panics if unset/non-absolute (contract
  violation by caller).
- **`ensure_pwd()`**: at top of `main`, snaps `$PWD` to match cwd when stale.
  `git -C foo` exec'ing us directly chdirs but leaves `$PWD` pointing at the
  parent shell's cwd; this fixes it once, before anything else reads `$PWD`
  or spawns subprocesses.
- **`samefile(a, b)`**: inode (`dev`+`ino`) equality, Python's
  `os.path.samefile`. `ensure_pwd` uses it to decide "stale or not"; this
  preserves any logical/symlinked `$PWD` the user set up via `cd through-a-symlink`
  rather than clobbering it with `getcwd()`'s canonical form.
- **`repo_workdir(start)`**: walks textual parents from `start` looking for
  a `.git` entry of any shape (directory, symlink, gitfile). Replaces
  `repo.workdir()`.

## Why Not getcwd() or canonicalize

Both resolve symlinks against the filesystem's *current* state. On Linux,
`getcwd()` returns a canonical form, losing any logical path the user
navigated through. `canonicalize` captures whatever the link points at
right now; if the link target later moves, the captured path is stale.

`$PWD`-based resolution stays valid as long as the symlinks themselves do,
matches what shell tools display (`pwd`, `git rev-parse --show-toplevel`),
and makes textual `parent()` walks safe.

## Contract

- **CLI binaries** call `ensure_pwd()` at the top of `main` before any other
  code runs. Single env-mutation point per process.
- **Library functions** read `$PWD` via `logical_cwd()` and assume the caller
  has arranged correct values. Library consumers driving the API from
  arbitrary code must either call `ensure_pwd` themselves or pass absolute
  path arguments.
- Within the crate, no code reads `std::env::current_dir()` or calls
  `fs::canonicalize` on a workdir-relative path. The `workdir` module is the
  single point of contact for these primitives.

## Trade-off

When `$PWD` is stale and gets repaired, we substitute `getcwd()`'s canonical
form — losing logical symlink info. Accepted because we can't recover the
original `-C` argument (git consumed it before exec) and the alternative
(silently using a stale path) is worse.
