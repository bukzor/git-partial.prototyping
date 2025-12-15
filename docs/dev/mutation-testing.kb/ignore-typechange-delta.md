---
status: gap
attempts: 2
---

# Typechange File to Symlink

Staging a change from regular file to symlink (or vice versa) produces `Delta::Typechange`.

Current code hits the `_` arm and uses `new_file()` oid/mode. This might work, but the mode would be different (symlink mode vs regular file mode).

## Injection
Add `git2::Delta::Typechange => continue` to skip typechange deltas.

## Attempts
1. Added `commits_staged_typechange` test that converts file to symlink - passed with mutation

Mutation is unkillable - git2's `diff_tree_to_index` may report file-to-symlink as Modified, not Typechange. Test added and working, but can't verify via mutation testing.
