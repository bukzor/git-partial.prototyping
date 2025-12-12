---
status: gap
---

# Renames Only Delete

Git represents renames as delete + add. If we only process the old_file side of a rename delta, the new file wouldn't appear in the commit.

Inject by always using `delta.old_file().path()` instead of preferring `new_file().path()`.
