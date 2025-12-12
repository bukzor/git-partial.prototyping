---
status: gap
---

# Rename With Detection Loses Old File

If git's rename detection is enabled (via config or DiffOptions), a rename shows as a single `Delta::Renamed` instead of separate Add + Delete.

Current code:
1. Gets path from `new_file()` (the destination)
2. Hits the `_` arm (not Deleted), adds destination with content
3. Never removes the source

The old file would persist in the commit tree alongside the new one.

Test currently passes because `diff_tree_to_index(..., None)` doesn't enable rename detection. But if user has `diff.renames=true` in gitconfig, this could break.

Inject by passing DiffOptions with `opts.renames(true)` and verifying old file persists.
