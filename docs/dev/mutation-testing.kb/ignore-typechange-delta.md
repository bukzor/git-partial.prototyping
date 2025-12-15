---
status: equivalent
attempts: 3
---

# Ignore Typechange Delta

## Mutation
Add `git2::Delta::Typechange => continue` to skip typechange deltas.

## Finding: Not Applicable (Delta Type Not Observed)

libgit2's `diff_tree_to_index()` reports file-to-symlink changes as `Delta::Modified` or separate deltas, not `Delta::Typechange`. The `commits_staged_typechange` test passes regardless of any Typechange-specific handling.

No code exists to handle this delta type explicitly, and adding such code would be dead.

## Resolution

No code changes needed. The wildcard `_` match handles all observed delta types correctly.
