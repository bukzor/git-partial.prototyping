# Devlog: 2026-05-12 — Bypass libgit2 Workdir for Symlinked .git

## Focus

`git-commit-staged` and `git-commit-files` failed under the git-localhost-store
layout (absolute symlink for `.git`). libgit2's `repo.workdir()` followed the
symlink and returned the wrong workdir; downstream `starts_with` checks then
rejected every path with "outside repository".

## What Happened

### Discovered

- `git2::Repository::workdir()` returns `parent(canonical(gitdir))` for
  non-bare repos. Wrong when `.git` symlink target is outside the workdir.
- `std::fs::canonicalize` has the sibling defect of resolving symlinks
  against current FS state and producing paths that diverge from `$PWD`.
- `git -C foo` exec'ing a binary directly chdirs without updating `$PWD`.
  Shell-script subcommands "fix" this only because `/bin/sh` resets
  `$PWD = getcwd()` on startup; binaries inherit the stale value.

### Built

- New `src/workdir.rs`: `logical_cwd`, `ensure_pwd`, `samefile`,
  `repo_workdir`. Module-level docs capture the why.
- Replaced `repo.workdir()` + `fs::canonicalize` in `src/exec.rs` and
  `src/prepare.rs` with `logical_cwd()` + `repo_workdir()`. `prepare.rs`
  uses `gix_path::normalize` for textual `.`/`..` cleanup.
- `ensure_pwd()` at top of `main` in both binaries. Uses
  `samefile(".", $PWD)` to avoid clobbering a still-valid logical path
  with `getcwd()`'s canonical form.
- Two regression tests, one per bypass site:
  - `git_integration.rs::commits_through_symlinked_dotgit` exercises
    `prepare_staged_commit` (library path).
  - `integration_files.rs::commits_through_symlinked_dotgit` exercises
    the `commit-files` binary end-to-end (`stage_paths_to_temp` in
    `exec.rs`).
- Extracted shared test helpers to `tests/testing/mod.rs`. `git_init`
  overrides `init.templateDir=` so tests are hermetic against developer-local
  git-localhost-store auto-migration hooks (which would otherwise relocate
  test repos' `.git` into the central store and leak state across runs).

### Renamed mid-session

User flagged "sync-pwd" as a poor name. Reframed as "ensure the environment
is set the way it should be":

- `sync_pwd` → `ensure_pwd`
- Inner `pwd_matches` → `pwd_names_cwd`; switched from
  canonicalize-vs-canonicalize compare to inode (`dev`+`ino`) equality via
  extracted `samefile` helper (Python's `os.path.samefile`; Rust std has
  no equivalent and the `same-file` crate isn't worth a dep for one
  stat compare).

## Decisions Made

### Use `$PWD` as the crate's source of truth for cwd

**Rationale**: stable under symlinks, matches what shells show, makes textual
`parent()` walks safe.

**Alternatives rejected**:
- `core.worktree` per gitdir: defeats git-localhost-store's "workdir moves
  freely" property.
- `fs::canonicalize`: captures stale paths if symlinks later move, and
  diverges from logical `$PWD`.

### Single env-mutation point

`ensure_pwd` is the only writer of `$PWD`, called once at the top of `main`.
Library code reads via `logical_cwd()` only. Documented in the pattern doc
as a contract.

## Links

- New constraint: `docs/dev/design.kb/discovered-constraints.kb/libgit2-workdir-misidentifies-symlinked-gitdir.md`
- New pattern: `docs/dev/design.kb/integration-patterns.kb/logical-pwd-path-resolution.md`
- Previous devlog: `2026-02-04-001-homebrew-formula-self-hosting.md`
