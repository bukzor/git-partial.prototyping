# Current Work

## Mutation Testing Gap Closure

- [x] Close remaining `status: gap` mutations in `docs/dev/mutation-testing.kb/`
  - [x] `skip-index-write` - removed dead `unstage_paths` function, added `git_status_clean_after_commit` test
  - [x] Update 5 equivalent mutations to `status: equivalent`
    - `enable-rename-detection` - Delta::Renamed never observed from diff_tree_to_index
    - `filter-rename-by-destination-only` - renames appear as separate Delete+Add deltas
    - `ignore-typechange-delta` - Delta::Typechange never observed
    - `use-only-new-file-path` - removed dead `.or_else()` fallback
    - `use-only-old-file-path` - git2 populates new_file().path() for all deltas
- [ ] Port remaining CLI error tests to git-integration where beneficial
