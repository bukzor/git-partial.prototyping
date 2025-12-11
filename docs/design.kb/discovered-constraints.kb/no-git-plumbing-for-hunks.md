---
solutions:
  - ../technology-choices.kb/diff-parsing-library.md
---

# No Git Plumbing for Hunks

Git's plumbing commands operate at file/blob level, not hunk level.

## What Exists

- `git diff-files` - compares working tree to index (file-level)
- `git diff-index` - compares tree to index/working tree (file-level)
- `git diff-tree` - compares two trees (file-level)

## Raw Output Format

```
:100644 100644 bcd1234 0123456 M file.c
```

Provides blob hashes, not hunk boundaries.

## Implication

Unified diff format (`git diff -p`) is the lowest level for change content. Hunk extraction requires parsing the unified diff output.

## Source

Git documentation: `gitdiffcore(7)`, `git-diff-files(1)`
