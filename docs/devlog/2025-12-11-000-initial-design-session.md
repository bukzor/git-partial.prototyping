# 2025-12-11: Initial Design Session

## Focus

Design git-partial: chunk-wise staging for Claude Code agents, emulating `git commit -p` non-interactively.

## Journey

### Problem Exploration
Started with: "Claude Code can't use `git commit -p`". Interactive commands don't work in Bash() tool.

### Baseline Rejected
Considered manual approach: move working copy to tmp, restore HEAD, recreate hunks via Edit tool. Too token-expensive - requires re-reading and re-editing files.

### Key Discovery: Isolated Index
`git apply --cached` stages changes WITHOUT touching working copy. Combined with `GIT_INDEX_FILE` env var, enables fully isolated partial commits:
- Main index untouched (other agents' staged work preserved)
- Working copy untouched (uncommitted changes remain)
- Hunk-level granularity via patch application

### Git Plumbing Research
Fetched actual git documentation from GitHub. Confirmed: no hunk-level plumbing exists. `git diff-files`, `git diff-index`, `git diff-tree` all operate at file/blob level. Unified diff format IS the lowest level for change content.

### Hunk Extraction
Found `filterdiff` from patchutils - canonical tool for patch manipulation. But it's not on PyPI/cargo, only system package managers. Pivoted to native libraries:
- Python: `unidiff` (excellent)
- Rust: `patch` crate (484k downloads)

### Technology Decisions
- Rust implementation (user's learning goal)
- git-absorb as architectural model (small, focused, uses git2)
- Environment-agnostic design (eval-able output, no Claude Code coupling)
- Nested workspace layout (internal crates inside git-partial/)

### Detours
- Fixed `claude-workspace-merge`: removed `xargs -o` flag (no tty available)
- Fixed `llm.kb-validate`: changed `.d` → `.kb` suffix, added SUFFIX constant

## Decisions

1. Use `GIT_INDEX_FILE` for isolated index operations
2. Use `.git/hunks.d/{file}/{start-end}.patch` for hunk selection
3. Implement in Rust with `patch` crate
4. Emit eval-able output (ssh-agent pattern)
5. Nested workspace: internal crates inside git-partial/
6. Subsystems: cli, export, git, hunks, patch, session

## Artifacts

- `docs/design.kb/` with 17 validated files
- CLAUDE.md declaring llm.kb + llm-collab dependencies
- No Rust code yet - design phase complete

## Next Session

1. Initialize Cargo workspace with nested layout
2. Stub out crate structure (git-partial/, claude-export/)
3. Start with `patch` crate - parse diff, extract hunks
4. Or: prototype the workflow in shell first to validate
