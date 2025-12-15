---
status: gap
attempts: 2
---

# Skip Index Write After Commit

If `unstage_paths` prepares the index but doesn't call `index.write()`, the main index would still show committed paths as staged after the commit completes.

## Mutation Injection
Remove the `index.write()` call at the end of `unstage_paths` (line 273).

## Testing Challenge
Created test `committed_files_unstaged` that stages two files, commits one via the tool, and verifies only the committed file is unstaged. Test passes even with mutation in place. Possible causes:
- Git's own commands (`git status`, `git ls-files`) may reload the index from disk internally
- libgit2's behavior when the index isn't written may differ from git CLI
- Test environment specifics

Requires Opus-level investigation into why the mutation is unkillable through available git operations.
