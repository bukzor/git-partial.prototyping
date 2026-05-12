---
solutions:
  - ../integration-patterns.kb/logical-pwd-path-resolution.md
---

# libgit2 Workdir Misidentified When .git Is a Symlink

`git2::Repository::workdir()` (libgit2's `git_repository_workdir`) infers the
working directory as `parent(canonical(gitdir))` for non-bare repos with no
`core.worktree` set. When `.git` is a symlink to an absolute path elsewhere on
disk, canonicalize follows the symlink and the inferred parent is **not** the
real working tree.

## Triggering Layout

git-localhost-store (`~/.local/share/git-localhost-store/`) maintains every
workdir's `.git` as a symlink into a path-encoded central store:

    <workdir>/.git → ~/.local/state/git-localhost-store/repos/<encoded>/

libgit2 returns `~/.local/state/git-localhost-store/repos` as the workdir.
Path-resolution checks (`starts_with`, `strip_prefix` against this wrong root)
then reject every path with "outside repository".

## Mitigation Considered and Rejected

Setting `core.worktree` per gitdir silences libgit2 but couples each gitdir to
a fixed workdir path, defeating the layout's purpose (workdir moves freely;
only the symlink updates).

## Related: std::fs::canonicalize Has a Sibling Defect

`canonicalize` resolves symlinks against the filesystem's *current* state. The
result captures whatever the link points at right now; if the link target
later moves, the captured path is stale. Same reason rules it out as a
naive workaround for the libgit2 misbehavior.

## Affected Call Sites

Any code calling `repo.workdir()` in a context that may run under this layout.
In this crate: `src/exec.rs::stage_paths_to_temp` and
`src/prepare.rs::prepare_staged_commit`.
