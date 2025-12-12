---
status: gap
---

# New Files Fail Unstage

After committing a new file, `unstage_paths` tries to reset the index entry to match HEAD. But the logic reads from `head_tree.get_path()` which should work (the file is now in HEAD after our commit). However, if we accidentally used the old HEAD reference, new files wouldn't exist there.

Inject by caching HEAD tree before creating the commit and using that stale reference in unstage.
