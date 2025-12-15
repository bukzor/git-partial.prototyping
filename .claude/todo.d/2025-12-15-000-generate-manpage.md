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

Use `clap_mangen` crate to generate man page from existing clap definitions at build time.

## Implementation Steps

- [ ] Add `clap_mangen` as build dependency
- [ ] Create `build.rs` to generate man page
- [ ] Add install target for man page (cargo doesn't handle this natively)
- [ ] Document manual installation in README

## Open Questions

- Where should man page install to? (`~/.local/share/man/man1/`? system `/usr/share/man/man1/`?)
- Should this be part of `cargo install` or separate step?

## Success Criteria

- [ ] `git commit-staged --help` displays man page
- [ ] Installation documented

## Notes

Low priority — `-h` works fine as workaround. Nice-to-have for 1.0 polish.
