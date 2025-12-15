---
status: equivalent
attempts: 3
---

# Use Only New File Path for Deltas

## Mutation
Remove the `.or_else(|| delta.old_file().path())` fallback.

## Finding: Equivalent (Dead Code)

The fallback was never executed. git2's `diff_tree_to_index` populates `new_file().path()` for all delta types, including deletions.

## Verification

Removed the fallback; all tests pass including `commits_staged_deletion`.

## Resolution

Removed dead `.or_else()` fallback. Added comment explaining git2 behavior.
