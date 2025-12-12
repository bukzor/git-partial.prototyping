---
status: gap
---

# Hardcode Filemode in Commit

If `create_commit` ignores the staged filemode and always uses 0o100644, executable bits would be lost even when `chmod +x` was staged.

Inject by replacing `*mode` with `0o100644` in the IndexEntry construction.
