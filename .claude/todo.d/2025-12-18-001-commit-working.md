<anthropic-skill-ownership llm-subtask />

# Add git-commit-working sibling command

**Priority:** Medium
**Complexity:** Low
**Context:** User friction in `git add paths && git commit-staged paths` pattern

## Problem Statement

Common workflow requires duplicating paths between `git add` and `git commit-staged`:

```bash
git add src/foo.py src/bar.py
git commit-staged src/foo.py src/bar.py -- -m "message"
```

This is error-prone and verbose.

## Proposed Solution

Add sibling command `git-commit-working` that stages then commits from working tree:

```bash
git commit-working src/foo.py src/bar.py -- -m "message"
```

Two distinct tools, two distinct operations:
- `commit-staged` — commit what's already in the index at these paths
- `commit-working` — stage then commit these paths from working tree

## Implementation Steps

- [x] Create `git-commit-working` binary (new crate or extend existing)
- [x] Stage specified paths before calling commit logic
- [x] Share commit machinery with `git-commit-staged`
- [x] Add tests mirroring `git-commit-staged` test patterns
- [x] Update README with both commands

## Success Criteria

- [x] `git commit-working paths... -- -m "msg"` works end-to-end
- [x] Unstaged changes at paths are staged then committed
- [x] Already-staged changes at paths are preserved
- [x] Index hygiene: only specified paths affected
