---
status: done
---

# Use Wrong Parent Commit in create_commit

If `create_commit` uses an incorrect parent commit when creating the new commit, the commit graph would be broken or the commit would have wrong ancestry.

## Injection

Replace `&[&head_commit]` with grandparent (or empty for initial commit) in `repo.commit()` call (line 225).

## Test Coverage

10 integration tests fail with "current tip is not the first parent":
- `commits_staged_file_at_specified_path`
- `commits_staged_deletion`
- `commits_staged_mode_change`
- `commits_staged_rename`
- `commits_rename_with_detection_enabled`
- `commits_staged_typechange`
- `only_commits_staged_not_working_copy`
- `preserves_other_staged_files`
- `respects_directory_scope`
- `committed_files_unstaged`

Git refuses to update HEAD when the proposed commit's parent doesn't match current tip.
