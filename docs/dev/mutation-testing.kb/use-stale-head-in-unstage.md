---
status: done
---

# New Files Fail Unstage

After committing a new file, `unstage_paths` tries to reset the index entry to match HEAD. But the logic reads from `head_tree.get_path()` which should work (the file is now in HEAD after our commit). However, if we accidentally used the old HEAD reference, new files wouldn't exist there.

## Injection
Cache HEAD tree before `create_commit`, pass stale tree to `unstage_paths`.

## Test Coverage
`commits_staged_file_at_specified_path` fails with "committed file not found in new HEAD" because new files don't exist in the stale HEAD tree.
