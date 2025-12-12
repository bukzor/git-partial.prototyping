---
status: gap
---

# Index Conflicts Unhandled

During a merge conflict, the index has entries at stages 1, 2, 3 (base, ours, theirs) instead of stage 0.

`diff_tree_to_index` behavior with conflicted entries is unclear. The tool might:
- Crash
- Silently ignore conflicts
- Commit one side of the conflict

Should probably error clearly: "cannot commit-staged with unresolved conflicts".
