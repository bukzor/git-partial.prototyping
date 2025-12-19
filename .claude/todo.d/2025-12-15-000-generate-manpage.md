<anthropic-skill-ownership llm-subtask />

# Generate manpage for git-commit-staged

**Priority:** Low (pre-1.0, not blocking)
**Complexity:** Low
**Context:** `docs/dev/milestones.kb/040-polish.md` — fits with "Shell completion scripts"

## Problem Statement

`git commit-staged --help` invokes `git help`, which looks for a man page. Currently shows "No manual entry" instead of the clap help text. Users must use `-h` instead.

## Current Situation

- `-h` works (clap short help)
- `--help` triggers git's man page lookup, fails
- No man page exists

## Proposed Solution

Use `clap_mangen` crate to generate man page at build time, commit to repo, distribute via Homebrew tap.

**Why this approach:** Cargo doesn't support installing man pages (rust-lang/cargo#2729, open since 2016). The ecosystem convention is to generate and commit man pages, then let package managers (Homebrew, apt, etc.) install them properly.

## Implementation Steps

- [x] Add `clap_mangen` as build dependency
- [x] Create `build.rs` to generate man page to `man/git-commit-staged.1`
- [x] Commit generated man page to repo (follows dust, broot convention)
- [ ] Add formula to `bukzor/tap` Homebrew tap
- [x] Document in README: symlink to ~/.local/share/man for cargo install users

## Open Questions

~~Where should man page install to?~~ **Resolved:** Homebrew handles this. Cargo install users do manual cp.

~~Should this be part of `cargo install`?~~ **Resolved:** No, cargo can't do this. Use Homebrew.

## Success Criteria

- [ ] `brew install bukzor/tap/git-commit-staged` installs binary + man page
- [ ] `git commit-staged --help` displays man page (for brew users)
- [ ] README documents both install methods

## Effort Estimate

~1-1.5 hours total:
- clap_mangen + build.rs: 30 min
- Generate and commit man page: 10 min  
- Homebrew formula: 20 min
- Testing: 15 min

## Notes

Low priority — `-h` works fine as workaround. Nice-to-have for 1.0 polish.
