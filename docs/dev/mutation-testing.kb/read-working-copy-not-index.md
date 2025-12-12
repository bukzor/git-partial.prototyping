---
status: done
---

# Working Copy Leaks Into Commit

The core guarantee: commit staged content, not working copy. If `create_commit` accidentally read file content from disk instead of using the staged blob OID, working copy changes would leak.

Current code uses `f.id()` from the diff delta (the staged blob OID). Inject by reading from disk and hashing instead.
