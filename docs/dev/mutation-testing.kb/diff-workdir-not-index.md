---
status: done
---

# Diff Working Directory Instead of Index

If `find_staged_entries` diffs HEAD against working copy instead of index, uncommitted working copy changes would leak into the commit even when they're not staged.

Inject by changing `diff_tree_to_index` to `diff_tree_to_workdir`.
