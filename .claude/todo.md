# Current Work

- [x] `commit-files` should handle untracked files — fixed by `update_all` + `add_all` (b32f9f9)
- [x] `commit-files` fails on deleted files — replaced shell-out `git add` with git2 `update_all` + `add_all`
- [x] `commit-files`: bail if index differs from both HEAD and working tree (three-version case)
  - Prevents silent destruction of staged changes
  - Helpful error message suggesting `git commit-staged` or `git reset`

# Future

- [ ] Locking for multi-agent scenarios
  - Hold `.git/index.lock` during check + commit for atomicity
  - Use `git hook run` + `git commit-tree` instead of `exec git commit`
  - Benefits both `commit-files` and `commit-staged`

- [x] Close remaining `status: gap` mutations in `docs/dev/mutation-testing.kb/`
  - [x] `skip-index-write` - removed dead `unstage_paths` function, added `git_status_clean_after_commit` test
  - [x] Update 5 equivalent mutations to `status: equivalent`
    - `enable-rename-detection` - Delta::Renamed never observed from diff_tree_to_index
    - `filter-rename-by-destination-only` - renames appear as separate Delete+Add deltas
    - `ignore-typechange-delta` - Delta::Typechange never observed
    - `use-only-new-file-path` - removed dead `.or_else()` fallback
    - `use-only-old-file-path` - git2 populates new_file().path() for all deltas
- [x] Port CLI tests to git-integration where beneficial
  - Ported 12 tests from integration.rs to git_integration.rs (in-process)
  - Kept 3 CLI-specific tests: `respects_directory_scope`, `directory_scope_prevents_escape`, `dry_run_does_not_commit`
  - All ported tests validated via mutation testing
- [x] GHA workflow(s) to run all tests, clippy all code
  - Single CI workflow with test and lint jobs
  - Lint job auto-fixes fmt/clippy on PRs and pushes back
