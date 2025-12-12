---
status: gap
---

# Use Only New File Path for Deltas

If `find_staged_entries` only uses `new_file().path()` without falling back to `old_file().path()`, deletions would have no path (new_file is empty for deletions) and would error or be skipped.

Inject by removing the `.or_else(|| delta.old_file().path())` fallback.
