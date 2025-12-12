---
status: done
---

# Scope Escape via Symlinks

`canonicalize()` resolves symlinks. If user creates a symlink inside the scope that points outside, then specifies that symlink as a path, the resolved path would be outside scope but we might not catch it.

Current code does check `normalized.starts_with(scope_root)` after canonicalization, which should catch this. Verify by injecting a bug that skips the scope check.
