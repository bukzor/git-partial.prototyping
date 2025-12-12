---
status: done
---

# Skip Deletion Removal in Commit

If `create_commit` skips the `index.remove()` call for deleted files, the deletion wouldn't take effect - the file would persist in the committed tree even though it was staged for deletion.

Inject by changing the `None` arm in `create_commit` to a no-op instead of calling `index.remove()`.
