---
status: done
---

# Skip Scope Root Canonicalization

If we skip canonicalizing `scope_root` (or skip canonicalizing user paths after joining), the scope escape check could be bypassed with symlinks.

## Injection
Remove the `canonicalize()` call on line 35-36 and use `args.directory.clone()` directly.

## Test Coverage
Mutation causes 10 test failures with "prefix not found" - the path matching logic requires absolute/canonical paths to work correctly.
