---
status: equivalent
attempts: 3
---

# Skip Index Write After Commit

## Mutation
Remove the `index.write()` call at the end of `unstage_paths`.

## Finding: Equivalent (Dead Code)

The entire `unstage_paths` function was dead code. Investigation:

1. After staging, the index contains `(path, blob_oid, mode)`
2. After commit, HEAD contains the same `(path, blob_oid, mode)`
3. `diff_tree_to_index` compares OIDs — they match, so no diff
4. The file automatically appears "unstaged" because it matches HEAD

`unstage_paths` was overwriting index entries with identical OIDs — a no-op.

## Verification

- Skipping `unstage_paths` entirely: all tests pass
- Sabotaging it (removing files from index): `git_status_clean_after_commit` catches it
- The test can detect real corruption, confirming the original code was equivalent

## Resolution

Removed `unstage_paths` function and its call site. Added `git_status_clean_after_commit` test to verify post-commit index state.
