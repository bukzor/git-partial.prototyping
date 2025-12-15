---
status: done
---

# Skip Scope Root Canonicalization

If we skip canonicalizing `scope_root` (or skip canonicalizing user paths after joining), the scope escape check could be bypassed with symlinks.

## Injection
Remove the `canonicalize()` call on `directory` in `git_commit_staged()` and use `directory.to_path_buf()` directly.

## Test Coverage
Mutation causes 10 test failures with "prefix not found" - the path matching logic requires absolute/canonical paths to work correctly.
