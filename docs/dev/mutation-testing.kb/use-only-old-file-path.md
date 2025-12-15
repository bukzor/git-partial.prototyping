---
status: equivalent
attempts: 3
---

# Use Only Old File Path for Deltas

## Mutation
Replace `new_file().path()` with `old_file().path()`.

## Finding: Equivalent (Dead Code)

git2 populates both `old_file().path()` and `new_file().path()` with the same value for all delta types from `diff_tree_to_index`. The fallback to `old_file()` was never needed.

## Resolution

Removed dead `.or_else(|| delta.old_file().path())` fallback. Using `new_file().path()` directly.
