# Devlog: 2026-02-04 - Homebrew formula self-hosting

## Focus

Move Homebrew formula into source repo for simpler iteration.

## What Happened

### Completed

- Investigated Homebrew's `cargo` Brewfile predicate — just `cargo install --locked`, no man pages
- Moved formula from `bukzor/homebrew-tap` to `Formula/git-commit-staged.rb`
- Fixed license: MIT → Apache-2.0 (matching Cargo.toml)
- Added LICENSE file
- Symlinked local Homebrew tap to source repo for instant formula iteration
- Updated and marked manpage todo complete

### Discovered

- Homebrew formulas like ripgrep.rb explicitly call `man1.install` — not automatic
- `brew reinstall` remembers original `--HEAD` flag
- Self-hosting formula avoids chicken-and-egg SHA256 problem for HEAD installs

## Decisions Made

### Self-host formula in source repo

**Rationale:** Simpler workflow — one push updates both formula and source. No separate tap repo to maintain.

**Alternatives:** Separate `bukzor/homebrew-tap` repo (previous approach)

**Impact:** Install command becomes `brew tap bukzor/git-partial https://github.com/bukzor/git-partial.prototyping` (longer but one-time)

## Links

- Previous devlog: 2026-02-04-000-multi-agent-locking.md
- Todo closed: .claude/todo.d/2025-12-15-000-generate-manpage.md
