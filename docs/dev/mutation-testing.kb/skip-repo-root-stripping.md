---
status: done
---

# Skip Repo Root Stripping

If `resolve_paths` doesn't strip `repo_root` from the normalized path, the paths used for index operations would be absolute instead of repo-relative.

Index operations expect repo-relative paths. Absolute paths would fail to match entries in the index, causing the tool to report "no staged changes at specified paths" even when changes exist.

## Injection

Remove `strip_prefix(repo_root)` call (lines 115-124), return `normalized.to_path_buf()` directly.

## Test Coverage

11 integration tests fail with "no staged changes at specified paths":
- `commits_staged_file_at_specified_path`
- `commits_staged_deletion`
- `commits_staged_mode_change`
- `commits_staged_rename`
- `commits_rename_with_detection_enabled`
- `commits_staged_typechange`
- `only_commits_staged_not_working_copy`
- `preserves_other_staged_files`
- `respects_directory_scope`
- `dry_run_does_not_commit`
- `committed_files_unstaged`
