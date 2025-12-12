---
status: not-started
goal: Multiple concurrent agents maintain separate sessions across tool calls
depends-on: ["010"]
deliverables:
  - "Session IDs (UUIDs) for isolation"
  - "git partial list - enumerate active sessions"
  - "git partial drop <id> - cleanup session"
  - "git partial export - emit eval-able environment variables"
  - "Sessions persist in .git/partial.d/{uuid}/"
---

# Milestone 020: Multi-Agent Persistence

## Goal

Enable multiple Claude Code agents to work concurrently in the same repository, each maintaining separate partial-commit state.

## Scope

**In scope:**
- Generate UUID per session
- Store session state in `.git/partial.d/{uuid}/`
- `git partial export` emits eval-able vars (ssh-agent pattern)
- Session discovery and enumeration
- Session cleanup

**Out of scope:**
- Conflict resolution between sessions
- Session sharing/merging
- Garbage collection of stale sessions

## Success Criteria

Can execute this workflow across multiple Claude Code agent sessions:

**Agent 1:**
```bash
eval $(git partial init)
# $GIT_PARTIAL_SESSION set
rm $GIT_PARTIAL_HUNKS/foo/10-20.patch
git partial commit -m "agent 1 work"
```

**Agent 2 (concurrent):**
```bash
eval $(git partial init)
# Different $GIT_PARTIAL_SESSION
rm $GIT_PARTIAL_HUNKS/bar/5-10.patch
git partial commit -m "agent 2 work"
```

**User:**
```bash
git partial list    # shows both sessions
git partial drop <uuid>  # cleanup
```

## This Solves

Dotfiles repo at `~` with multiple concurrent agent sessions. Each agent commits its own changes without interfering with others.
