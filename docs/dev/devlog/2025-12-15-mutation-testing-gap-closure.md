---
date: 2025-12-15
session: Mutation Testing Gap Closure
type: devlog
---

# Mutation Testing Gap Closure: git-integration Test Infrastructure

## Focus

Close mutation testing gaps by adding in-process tests that can inspect error types directly, rather than relying on CLI stderr string matching.

## What Happened

### Empirical Investigation

Ran diagnostic tests to verify libgit2/git2 behavior claims in gap documentation:

1. **Delta path accessors**: Both `old_file().path()` and `new_file().path()` return paths for ALL delta types (Added, Deleted, Modified). The `or_else` fallback is never exercised.

2. **Rename detection**: Without `find_similar()`, renames appear as separate Add+Delete deltas. With `find_similar()`, they appear as `Delta::Renamed` with different paths in old/new.

3. **Typechange**: File-to-symlink appears as Add+Delete, NOT `Delta::Typechange`, without `find_similar()`.

4. **Conflicts**: `diff_tree_to_index` returns `Delta::Conflicted` for conflicted entries. Index has stages 1,2,3.

5. **Index write**: `index.write()` IS required - without it, changes aren't persisted to disk.

### Architecture Change

Split CLI binary from library to enable in-process testing:

- Created `src/lib.rs` with `git_commit_staged()` function
- `src/main.rs` now just parses args and calls the library
- New `tests/git_integration.rs` for in-process tests (git subprocess, self in-process)
- Existing `tests/integration.rs` remains for CLI integration (both subprocess)

### Gap Closed

**`ignore-index-conflict-stages`**: Changed from `gap` to `done`.

- Added `errors_on_merge_conflict_with_index_error` test in `git_integration.rs`
- Test verifies error chain contains `git2::Error` with `ErrorClass::Index`
- Mutation (skip Conflicted deltas) changes error to "no staged changes" - no git2 error in chain
- Test fails for the right reason when mutation injected
- Removed redundant CLI test that only checked stderr string

## Decisions

1. **Two test categories**: `integration.rs` (CLI behavior) vs `git_integration.rs` (error types, internal state). CLI tests verify user experience; git-integration tests verify implementation correctness.

2. **Error type checking over string matching**: For mutations that change error paths, string matching is fragile. Checking `git2::ErrorClass` directly catches the mutation reliably.

3. **Empirical verification required**: Don't trust gap documentation claims without running diagnostic code. Several "gaps" turned out to be equivalent mutations (dead code paths due to API behavior).

## Next Session

See `.claude/todo.md` for remaining work:

1. **`skip-index-write` gap**: Needs git-integration test that reads index via libgit2 after commit to verify write happened.

2. **5 equivalent mutations**: Mark with `status: equivalent` after empirical verification:
   - `enable-rename-detection` - dead code (no `find_similar()` call)
   - `filter-rename-by-destination-only` - git2 provides both paths
   - `ignore-typechange-delta` - typechange is Add+Delete
   - `use-only-new-file-path` - new_file has path for deletions
   - `use-only-old-file-path` - old_file has path for additions

## Files Changed

- `git-commit-staged/Cargo.toml` - added lib target
- `git-commit-staged/src/lib.rs` - new library with `git_commit_staged()`
- `git-commit-staged/src/main.rs` - now CLI wrapper only
- `git-commit-staged/tests/git_integration.rs` - new in-process tests
- `git-commit-staged/tests/integration.rs` - removed redundant conflict test
- `docs/dev/mutation-testing.kb/ignore-index-conflict-stages.md` - updated to `done`
