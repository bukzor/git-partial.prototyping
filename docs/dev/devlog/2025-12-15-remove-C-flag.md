# Remove -C Flag from git-commit-staged

## Context

Updating `~/.claude/must-read.d/before/git/commit.md` to document `git-commit-staged` revealed a conflict: `-C` in `git commit` means "reuse commit message from commit", but in `git-commit-staged` it meant "run in directory".

## Discovery

Tested whether `git -C` works for external subcommands:

```bash
cd /tmp && git -C /path/to/repo commit-staged -n -m "test" .
# Works - git changes CWD before invoking external subcommand
```

## Decision

Remove `-C`/`--directory` from `git-commit-staged`. Users invoke via `git -C <dir> commit-staged` instead.

Benefits:
- No collision with `git commit -C` semantics
- Simpler tool with fewer options
- Consistent with how git subcommands work

## Changes

1. `git-commit-staged/src/main.rs`: Removed `-C` arg, hardcode `Path::new(".")`
2. `git-commit-staged/tests/integration.rs`: Updated helper to invoke via `git -C <dir> commit-staged`
3. `docs/dev/mutation-testing.kb/skip-scope-canonicalization.md`: Updated injection instructions
4. `~/.claude/must-read.d/before/git/commit.md`: Document `git commit-staged` as default
5. `~/.claude/must-read.d/before/git/all-operations.md`: Updated index hygiene guidance

## Non-changes

The `directory` parameter remains in `git_commit_staged()` lib function - tests need to pass temp directories. Scope-escape checking (`../` rejection) remains valid and necessary.
