---
status: not-started
goal: Can commit selected hunks from dirty working copy, once
depends-on: []
deliverables:
  - "git partial init - parse git diff, write .git/hunks.d/{file}/{start-end}.patch"
  - "git partial commit -m 'msg' - apply hunks via isolated index, commit via plumbing"
  - "Validation: main index + working copy untouched after commit"
---

# Milestone 010: Single-Session Core

## Goal

Prove the isolated index workflow. User can commit selected hunks from a dirty working copy.

## Scope

**In scope:**
- Parse `git diff` output into individual hunk files
- User manually deletes unwanted hunk files
- Apply remaining hunks to isolated index
- Commit via git plumbing (write-tree → commit-tree → update-ref)
- Verify main index and working copy remain untouched

**Out of scope:**
- Multi-agent session management
- Error recovery
- Conflict detection
- Session persistence across tool calls

## Success Criteria

Can execute this workflow:
```bash
# Working copy has changes
git partial init           # creates .git/hunks.d/ with hunk files
rm .git/hunks.d/foo/10-20.patch  # manually exclude hunk
git partial commit -m "partial commit"  # commits remaining hunks
git diff --cached          # main index still clean
git diff                   # working copy still has uncommitted changes
```

## This Solves

Immediate need: commit specific hunks from dirty working copy without interactive `git commit -p`.
