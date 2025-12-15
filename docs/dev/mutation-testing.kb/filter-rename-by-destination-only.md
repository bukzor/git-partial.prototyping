---
status: equivalent
attempts: 3
---

# Filter Rename by Destination Only

## Mutation
For renames, only filter by destination path, ignoring source path.

## Finding: Equivalent (Unreachable Code Path)

libgit2's `diff_tree_to_index()` never returns `Delta::Renamed` — renames appear as separate Delete + Add deltas. Each delta has its own path in `new_file().path()`.

The scenario "rename across scopes" is handled correctly:
- Delete delta for `src/a.rs` (filtered by `src/` scope)
- Add delta for `tests/b.rs` (filtered by `tests/` scope)

No rename-specific filtering logic exists or is needed.

## Resolution

No code changes needed. Renames are naturally handled as separate deltas.
