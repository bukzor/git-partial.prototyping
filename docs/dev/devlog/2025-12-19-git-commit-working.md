# 2025-12-19: Add git-commit-working

## Focus

Implement `git-commit-working` sibling command per `.claude/todo.d/2025-12-18-001-commit-working.md`.

## What Happened

1. **Implemented git-commit-working** - stages paths from working tree then commits, combining `git add` + `git commit-staged` into one atomic operation.

2. **Refactored for symmetry** - restructured codebase so neither command is privileged:
   ```
   src/
     lib.rs           # types, re-exports
     prepare.rs       # core prepare logic (162 LOC)
     index.rs         # temp index creation (66 LOC)
     exec.rs          # shared CLI helpers (58 LOC)
     staged/          # git-commit-staged binary
       cli.rs
       main.rs
     working/         # git-commit-working binary
       cli.rs
       main.rs
     tests/           # unit tests
   ```

3. **Tests** - 33 total (28 original + 5 new for git-commit-working). All pass, clippy clean.

4. **Man pages** - both commands have man pages. Changed install location from `/usr/local/share/man` to `~/.local/share/man` (XDG-compliant, no sudo). README updated to use symlinks to avoid drift.

## Decisions

- **Module structure over flat files** - `src/staged/` and `src/working/` directories rather than `src/main_staged.rs`, `src/main_working.rs`. Cleaner, more idiomatic.
- **Symlinks for man pages** - `ln -s` instead of `cp` keeps them in sync with rebuilds.
- **~/.local/share/man** - follows XDG spec, auto-discovered by `man` if `~/.local/bin` is in PATH.

## Commits

- `3fc2418` Add git-commit-working sibling command
- `36f9da1` Use ~/.local/share/man and symlinks for man pages

## Loose Ends

- `.claude/todo.d/2025-12-15-000-generate-manpage.md` has uncommitted edits and is partially complete (man pages work, but Homebrew formula not done)
- `.claude/todo.d/2025-12-18-001-commit-working.md` is complete, could be archived

## Next Session

- Decide: close manpage todo as "good enough" or finish Homebrew formula
- Consider archiving completed todo files
