# 2026-02-04: Multi-agent locking

## Focus

Implement `.git/index.lock` acquisition to serialize commit-files/commit-staged operations in multi-agent scenarios.

## Problem

`git2::Index::write()` internally creates `index.lock`. If we hold `index.lock` ourselves for atomicity, calling `write()` on the real index fails with "index is locked".

## Solution

All index writes go to temp paths, then rename to real index:

1. **`IndexLock`** (`lock.rs`) - RAII guard for `.git/index.lock`
2. **`do_commit`** (`commit.rs`) - spawns `git commit` as subprocess (not exec) so we retain control to clean up lock
3. **`stage_paths_to_temp`** (`exec.rs`) - stages working tree to temp index, returns path + entries; caller decides to rename (commit) or delete (dry-run)

Unified flow ensures dry-run uses same staging code as real execution.

## Changes

- `lock.rs` (new): `IndexLock` RAII guard
- `commit.rs` (new): `do_commit` with subprocess
- `exec.rs`: `stage_paths_to_temp`, `commit_staged_index`, `discard_staged_index`
- `files/main.rs`: uses new unified flow
- `staged/main.rs`: acquires lock, uses `do_commit`
- `index.rs`: added `write_temp_index_for_paths` convenience wrapper
- `integration_files.rs`: `TestRepo` fixture asserts no lock/temp files on drop; new `fails_when_lock_is_held` test

## Tests

39 total (was 31). All pass, clippy clean.

## Follow-up (same session)

- Fixed subdirectory path resolution in `stage_paths_to_temp`
- Added `--version` flag with embedded git hash (e.g., `0.1.0 (a105291)`)
- 42 tests total
