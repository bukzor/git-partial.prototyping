---
status: done
---

# Equality Only Path Matching

If `path_matches` only checks exact equality and removes the `starts_with` check, directory-based matching would not work - only exact file paths could be committed.

## Injection

Change `entry_path.starts_with(p)` to `entry_path == p` in `path_matches` (line 293).

## Test Coverage

4 unit tests catch this mutation:
- `directory_contains_file` - "src" should match "src/main.rs"
- `nested_directory_contains_file` - "src/foo" should match "src/foo/bar/baz.rs"
- `multiple_requested_paths` - directory matching within multiple paths
- `root_matches_everything` - empty path should match all files
