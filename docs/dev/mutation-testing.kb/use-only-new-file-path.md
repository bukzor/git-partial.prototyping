---
status: gap
attempts: 2
---

# Use Only New File Path for Deltas

If `find_staged_entries` only uses `new_file().path()` without falling back to `old_file().path()`, deletions would have no path (new_file is empty for deletions) and would error or be skipped.

## Injection
Remove the `.or_else(|| delta.old_file().path())` fallback on line 148.

## Attempts
1. Ran existing `commits_staged_deletion` test - passed
2. Added `ls-tree` check to verify file gone from HEAD - passed

Mutation survives because git2's `new_file().path()` apparently returns a path even for deletion deltas. The fallback may be defensive code for edge cases not covered by current tests.
