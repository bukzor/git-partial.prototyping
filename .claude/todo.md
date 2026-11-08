---
status: open
cost-benefit-sweh:
  timebox:
    '@value': 0
    rationale: 'zero residual: all inline items done; sole open item is the child todo.d ref, rated separately'
    confidence: confident
  benefit-2w:
    '@value': 0
    confidence: confident
---
# Current Work

- [ ] [todo.d/2026-06-27-000-initial-commit-and-stdin-message.md](todo.d/2026-06-27-000-initial-commit-and-stdin-message.md) — `commit-files`/`commit-staged` can't make a repo's initial commit (unborn branch), and `-F -` silently yields an empty message
- [ ] Dry-run mislabels an add as a modification: `commit-files -n` and
      `commit-staged -n` both print `M` for a path the index holds as
      `A`. Observed 2026-09-10 committing a new file in `bukzor/dotfiles`
      (`git status` showed `??`, then `A` after staging; both dry-runs
      said `M`). The dry-run's job is catching a wrong-scope commit
      before it happens, so a status letter that can't separate add from
      modify undercuts it.
- [ ] Make `cargo install` work on musl without flags: the default
      static link needs `libssl.a`/`libcrypto.a`/`libz.a`, which Alpine
      doesn't ship by default (hit 2026-11-08 installing on Alpine WSL;
      worked around with `-C target-feature=-crt-static`, now in
      README). Candidates: git2's `vendored-openssl` +
      `vendored-libgit2` features, or dropping `https`/`ssh` from git2
      entirely — the tool only touches local repos, so it may not
      need OpenSSL at all.
- [ ] Commit output prints the tool's name where git prints the branch:
      `[commit-staged 05ba082]`, `[commit-files 8c64d88]`. It reads as a
      branch named `commit-staged`, costing a verification round-trip to
      rule out. Print the real branch.
- [ ] Stale `.git/index.commit-staged.*` temp indexes: 9 here, 0 in
      three other repos that use these tools daily (`bukzor/dotfiles`,
      `bukzor.claude-sessions`, `bukzor-agent-skills`). All 9 date
      2025-12-23..2026-02-04, and ~8 `commit-staged` runs on
      2026-09-11 added none — so this is **not** an ongoing leak from
      the normal path, and the strays likely just need deleting.
      Cleanup does exist (`commit.rs:34`, `exec.rs:37` and `:241`,
      `lib.rs:106`), but every call is `let _ = remove_file(...)`,
      discarding the failure, and `exec_git_commit` cannot clean up on
      success at all: it `exec()`s, replacing the process image, so its
      removal at `exec.rs:37` is unreachable except on the error path.
      Weigh that hole on its own merits — these 9 are not evidence of
      it. (Corrected 2026-09-11; the earlier "accumulate" was ungrounded.)
- [x] Renamed `no-git-plumbing-for-hunks.md` → `no-plumbing-for-reading-hunks.md`,
      H1 with it. Body was corrected in 8c64d88; the name kept
      overclaiming, since `git apply --cached` does write at hunk grain.
      Zero inbound references, so nothing else moved.
- [x] Add `--version` flag with embedded git commit hash
- [x] `commit-files` should handle untracked files — fixed by `update_all` + `add_all` (b32f9f9)
- [x] `commit-files` fails on deleted files — replaced shell-out `git add` with git2 `update_all` + `add_all`
- [x] `commit-files`: bail if index differs from both HEAD and working tree (three-version case)
  - Prevents silent destruction of staged changes
  - Helpful error message suggesting `git commit-staged` or `git reset`
- [x] `commit-files`: support directory paths as input (5ad4370)
- [x] Locking for multi-agent scenarios — see devlog 2026-02-04

# Future

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
