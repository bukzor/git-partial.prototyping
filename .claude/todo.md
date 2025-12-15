# Current Work

## Mutation Testing Gap Closure

- [ ] Close remaining `status: gap` mutations in `docs/dev/mutation-testing.kb/`
  - [ ] `skip-index-write` - needs git-integration test reading index via libgit2
  - [ ] Remove dead code for 5 equivalent mutations, update to `status: equivalent`
    - `enable-rename-detection` - dead code, we don't call `find_similar()`
    - `filter-rename-by-destination-only` - git2 provides path in both accessors
    - `ignore-typechange-delta` - typechange appears as Add+Delete without `find_similar()`
    - `use-only-new-file-path` - `.or_else(|| delta.old_file().path())` never executes
    - `use-only-old-file-path` - `old_file().path()` fallback never needed
- [ ] Port remaining CLI error tests to git-integration where beneficial
