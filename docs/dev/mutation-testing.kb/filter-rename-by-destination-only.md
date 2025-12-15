---
status: gap
attempts: 2
---

# Partial Rename Across Scopes

Rename `src/a.rs` -> `tests/b.rs`, then commit only `src/`.

Expected: `src/a.rs` is deleted (the source half of the rename).

## Mutation Attempted

In `find_staged_entries` (main.rs:145-149), injected bug that removes fallback to `old_file().path()`:

```rust
let path = delta
    .new_file()
    .path()
    .context("diff delta has no path")?;
```

This should cause deletions to fail if their paths are only in `old_file()`.

## Test Result

**FAILED (test doesn't catch mutation)**

Attempted to create `deletion_across_scopes_respects_filter` test which:
- Commits a deletion from `src/` while leaving a separate addition in `tests/` staged
- Verifies deletion is included and addition is excluded

Test passed even with mutation injected. This means libgit2 provides the path in both `new_file()` and `old_file()` for Delete deltas, making the fallback untestable.

## Root Cause

The `or_else(|| delta.old_file().path())` fallback cannot be tested because:
- For Delete deltas, git2 provides the deleted path in `new_file().path()`
- The fallback is never actually used
- Removing it doesn't break any observable behavior

## Verdict: GAP

Cannot construct a test that fails with mutation removed. The fallback is defensive but untestable with current libgit2. Deferred to Opus.
