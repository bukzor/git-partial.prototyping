---
status: done
---

# Other Staged Files Committed

If path filtering is broken, staged files outside the specified paths would be included in the commit.

Inject by removing the `path_matches` filter or always returning true.
