---
status: gap
attempts: 2
---

# Rename With Detection Loses Old File

If git's rename detection is enabled, a rename could show as a single `Delta::Renamed` instead of separate Add + Delete.

Current code doesn't explicitly handle `Delta::Renamed`:
1. Would get path from `new_file()` (the destination)
2. Hits the `_` arm, adds destination with content
3. Never removes the source → old file persists in commit tree

## Mutation Attempted

In `find_staged_entries` (main.rs:157-163), injected explicit `Delta::Renamed` handling that only adds the destination:

```rust
git2::Delta::Renamed => {
    let f = delta.new_file();
    (path_str, Some((f.id(), u32::from(f.mode()))))
}
```

This should cause the old file to persist in the commit tree when a rename is detected.

## Test Result

**FAIL (test doesn't catch mutation)**

Added test `commits_rename_with_detection_enabled` which:
- Sets `diff.renames=true` in git config
- Creates a rename with `git mv`
- Verifies old file is deleted from HEAD

Test passed even with mutation injected because libgit2's `diff_tree_to_index()` does NOT return `Delta::Renamed` even when git config has `diff.renames=true`.

## Root Cause

- libgit2 rename detection is only supported for tree-to-tree diffs, not tree-to-index
- git2-rs `DiffOptions` has no `renames()` method to enable it explicitly
- Therefore, git2 returns Delete+Add deltas for renames, not `Delta::Renamed`
- The buggy code path is unreachable with current libgit2 API

## Verdict: GAP

This is a theoretical vulnerability that cannot be tested with current libgit2 capabilities. Deferred to Opus.
