---
status: equivalent
attempts: 3
---

# Rename Detection Delta Type

## Mutation
Add explicit `Delta::Renamed` handling that fails to remove the source file.

## Finding: Equivalent (Unreachable Code Path)

libgit2's `diff_tree_to_index()` never returns `Delta::Renamed`:
- Rename detection requires `find_similar()` which only works for tree-to-tree diffs
- git2-rs `DiffOptions` has no `renames()` method for tree-to-index
- Renames appear as separate Delete + Add deltas

The wildcard `_` match handles all actual delta types correctly. Any explicit `Delta::Renamed` handling would be dead code.

## Resolution

No code changes needed. The current wildcard match is correct for all observable delta types.
