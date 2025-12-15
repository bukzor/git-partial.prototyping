---
status: gap
attempts: 2
---

# Renames Only Delete

Git represents renames as delete + add. If we only process the old_file side of a rename delta, the new file wouldn't appear in the commit.

## Injection
Replace `new_file().path().or_else(...)` with just `old_file().path()` on line 145-148.

## Attempts
1. Ran all integration tests - passed
2. Hardened `commits_staged_file_at_specified_path` with exact path check - passed

Mutation survives because git2 apparently populates both old_file and new_file paths for all delta types, including new files and renames.
