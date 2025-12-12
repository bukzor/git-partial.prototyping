---
status: gap
---

# Skip Scope Root Canonicalization

If we skip canonicalizing `scope_root` (or skip canonicalizing user paths after joining), the scope escape check could be bypassed with symlinks.

Inject by removing the `canonicalize()` call on line 35-36 and using the raw `args.directory` path. Then a symlink inside the scope pointing outside could escape.
