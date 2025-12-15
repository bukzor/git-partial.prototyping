# Passthrough to git commit

**Priority:** Medium
**Complexity:** Medium
**Context:** Enables `-C`, `--amend`, `--fixup`, GPG signing, hooks, etc. for free

## Problem Statement

`git-commit-staged` reimplements commit creation via libgit2. This misses:
- `-C <commit>` (reuse message)
- `--amend`
- `--fixup <commit>`
- GPG signing (`-S`)
- Commit hooks
- Editor integration (`-e`)
- All future `git commit` features

## Proposed Solution

Change architecture: prepare a temp index, then exec `git commit` with `GIT_INDEX_FILE`.

### New flow

1. Parse paths (scope filtering stays in Rust)
2. Build tree: HEAD + staged changes at specified paths
3. Write tree to temp index file
4. Exec `git commit` with:
   - `GIT_INDEX_FILE=<temp>`
   - All non-path args passed through
5. Clean up temp index

### Argument handling

Paths don't start with `-`, so:
```bash
git commit-staged src/ tests/ -m "message"
git commit-staged src/ --amend
git commit-staged . --fixup abc123
git commit-staged src/ -C HEAD~1
```

Parse paths until first `-` arg, rest goes to `git commit`.

Or use explicit `--`:
```bash
git commit-staged src/ tests/ -- -m "message"
```

## Implementation Steps

- [ ] Refactor: extract tree-building to separate function
- [ ] Write tree to temp index file instead of in-memory
- [ ] Replace libgit2 commit creation with `exec("git", "commit", ...)`
- [ ] Pass through non-path args to git commit
- [ ] Handle cleanup on success/failure
- [ ] Update tests for new behavior
- [ ] Verify hooks run correctly
- [ ] Test with `--amend`, `--fixup`, `-C`

## Open Questions

- Temp index location: `$GIT_DIR/index.commit-staged.$$`? `/tmp`?
- Should dry-run (`-n`) still be ours, or pass through to `git commit --dry-run`?
- How to handle `git commit` failure (cleanup temp index)?

## Risks

- Behavior changes slightly (now runs hooks)
- Need to handle exec failure gracefully
- Temp file cleanup on signals

## Success Criteria

- [ ] `git commit-staged src/ --amend` works
- [ ] `git commit-staged src/ --fixup HEAD~3` works  
- [ ] `git commit-staged src/ -C HEAD~1` works
- [ ] Commit hooks run
- [ ] Existing tests still pass
