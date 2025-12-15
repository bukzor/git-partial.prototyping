---
status: done
attempts: 3
---

# Index Conflicts Unhandled

During a merge conflict, the index has entries at stages 1, 2, 3 (base, ours, theirs) instead of stage 0.

## Injection
Add `git2::Delta::Conflicted => continue` to skip conflicts in find_staged_entries.

## Test Coverage

`errors_on_merge_conflict_with_index_error` in `tests/git_integration.rs`

## Resolution

Created in-process test that:
1. Creates merge conflict via git subprocess
2. Calls `git_commit_staged()` in-process
3. Verifies error chain contains `git2::Error` with `ErrorClass::Index`

With mutation: skipping Conflicted causes "no staged changes" error (no git2::Error in chain) → test fails.
Without mutation: processing Conflicted entry causes git2 Index error → test passes.

The previous CLI-level test couldn't distinguish these because both result in non-zero exit, just different error messages.
