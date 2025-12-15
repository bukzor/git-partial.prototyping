---
status: done
---

# Hardcode Filemode in Commit

If `create_commit` ignores the staged filemode and always uses 0o100644, executable bits would be lost even when `chmod +x` was staged.

## Injection
Replace `*mode` with `0o100644` in the IndexEntry construction (line 192 in create_commit).

## Test Coverage
Test was initially insufficient - it only checked that a commit was created. Hardened test now checks that the file is actually executable after being checked out from the committed tree, using Unix file permissions with `mode & 0o111 == 0o111`.
