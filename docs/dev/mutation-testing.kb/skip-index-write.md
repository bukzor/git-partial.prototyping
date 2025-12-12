---
status: gap
---

# Skip Index Write After Commit

If `unstage_paths` prepares the index but doesn't call `index.write()`, the main index would still show committed paths as staged after the commit completes.

Inject by removing the `index.write()` call at the end of `unstage_paths`.
