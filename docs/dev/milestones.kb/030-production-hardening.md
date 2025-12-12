---
status: not-started
goal: Reliable enough for daily dotfiles use
depends-on: ["020"]
deliverables:
  - "Conflict detection (patch won't apply)"
  - "git partial abort <id> - cleanup failed commit"
  - "Pre-commit validation of patches"
  - "Help text and usage examples"
  - "Basic test suite"
---

# Milestone 030: Production Hardening

## Goal

Handle edge cases and failures gracefully. Tool is reliable enough for daily use in production dotfiles workflow.

## Scope

**In scope:**
- Detect when patches won't apply cleanly
- Abort mechanism for failed partial commits
- Validate patch syntax before applying
- User-facing documentation and help
- Test coverage for core workflows
- Friendly error messages

**Out of scope:**
- Interactive conflict resolution
- Performance optimization
- Integration with other tools
- Distribution packaging

## Success Criteria

**Error handling:**
```bash
git partial init
# modify working copy after init
git partial commit -m "msg"
# ERROR: patch doesn't apply (working copy changed)
# session state preserved for inspection
git partial abort   # cleanup
```

**Documentation:**
- `git partial --help` shows usage
- Examples for common workflows
- Clear error messages guide user to resolution

**Testing:**
- Happy path: init → commit succeeds
- Conflict: patch doesn't apply
- Cleanup: abort removes session state
- Multi-session: no collision

## This Solves

Edge cases that break workflow and create frustration. Makes tool trustworthy for daily use.
