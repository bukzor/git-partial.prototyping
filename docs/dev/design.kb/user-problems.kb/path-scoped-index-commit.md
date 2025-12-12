# Path-Scoped Index Commit

Commit staged changes at specific paths while preserving other staged work.

## Problem

Claude Code agents stage changes incrementally. When ready to commit a subset:
- `git commit -- paths` commits from **working copy**, not index
- `git commit` (no paths) commits **all** staged changes
- No git command commits "staged changes at these paths only"

## Use Case

```
Index state:
  staged: src/foo.rs (agent A's work)
  staged: src/bar.rs (agent B's work)
  staged: tests/test_foo.rs (agent A's work)

Agent A wants to commit src/foo.rs + tests/test_foo.rs only.
Agent B's staged work must remain staged.
```

## Solution

`git-commit-staged` - commits index contents at specified paths using isolated index workflow.

```bash
git commit-staged src/foo.rs tests/test_foo.rs -m "Add foo feature"
```

## Related

- `git-partial`: Commits working copy hunks (different source, same isolated index pattern)
