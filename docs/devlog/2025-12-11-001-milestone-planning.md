# 2025-12-11: Milestone Planning

## Focus

Define project milestones and establish "done enough" criteria for git-partial.

## Journey

### Orientation
Started with user request to "have a look around." Explored project structure, reviewed design.kb/, confirmed understanding of the problem space and proposed solution.

### Milestone Definition
User asked: "at what point should we call this thing 'done enough'?"

Proposed 4-phase approach building from minimal working system to production readiness:
- M1: Single-session core (prove concept)
- M2: Multi-agent persistence (solve stated problem)
- M3: Production hardening (make reliable)
- M4: Polish (optional improvements)

User agreed. Identified M2 as "done enough" target - solves the actual problem (multi-agent concurrent sessions in dotfiles).

### Structure Choice
User suggested `milestones.kb/` over `ROADMAP.md` for future agent coordination.

**Rationale:**
- Consistency with existing `.kb/` pattern (design.kb already established)
- Each milestone = separate file = independent queries
- llm.kb validation ensures schema consistency
- Glob/Grep-friendly for agent queries
- CLAUDE.md establishes maintenance contract

### Numbering Convention
User invoked "double expected number of digits" guideline, then requested multiplying by 10 for insertion room.

Final scheme: 3-digit multiples of 10 (010, 020, 030, 040)
- Expected: 4 milestones
- Digits needed: 1
- Doubled: 2 digits
- With 10x spacing: 010, 020, 030, 040
- **Headroom:** 9 insertions between any two milestones without renumbering

## Decisions

1. **Structure:** Use `milestones.kb/` with llm.kb pattern, not flat ROADMAP.md
2. **Numbering:** 3-digit multiples of 10 (010, 020, 030, 040)
3. **Schema:** Frontmatter with status, goal, depends-on, deliverables
4. **"Done enough":** Milestone 020 (multi-agent persistence) solves stated problem
5. **ROADMAP.md:** Keep as pointer to milestones.kb/ with quick overview

## Artifacts

Created:
- `docs/milestones.jsonschema.yaml` - Schema validation
- `docs/milestones.kb/CLAUDE.md` - Maintenance guide
- `docs/milestones.kb/010-single-session-core.md` - Happy path proof
- `docs/milestones.kb/020-multi-agent-persistence.md` - Session isolation
- `docs/milestones.kb/030-production-hardening.md` - Error handling
- `docs/milestones.kb/040-polish.md` - Optional improvements

Validated: All frontmatter passes llm.kb-validate (21 files, 0 errors)

Committed: 08c8ad7 "Add milestones.kb/ structure with 4 development phases"

## Milestone Details

### 010: Single-Session Core
**Goal:** Prove isolated index workflow
**Key deliverables:**
- `git partial init` - parse diff, write hunk files
- `git partial commit` - apply hunks, commit via plumbing
- Verify main index + working copy untouched

**Out of scope:** Multi-session, error recovery, persistence

### 020: Multi-Agent Persistence
**Goal:** Multiple concurrent agents maintain separate sessions
**Key deliverables:**
- Session IDs (UUIDs)
- `git partial list/drop` - session management
- `git partial export` - eval-able output (ssh-agent pattern)
- Sessions persist in `.git/partial.d/{uuid}/`

**This is "done enough"** - solves the user problem.

### 030: Production Hardening
**Goal:** Reliable for daily dotfiles use
**Key deliverables:**
- Conflict detection
- `git partial abort` - cleanup failures
- Help text + examples
- Basic test suite

### 040: Polish (Optional)
**Goal:** UX improvements and ecosystem integration
**Potential:** TUI, Claude Code skill, performance, packaging

## Next Session

Options:
1. **Start implementation:** Initialize Cargo workspace, stub crates
2. **Shell prototype:** Validate workflow in bash before writing Rust
3. **Refine design:** Dive deeper into specific subsystems

Recommend: Shell prototype to validate assumptions before committing to Rust implementation. This aligns with devlog entry 2025-12-11-000 line 58 suggestion.
