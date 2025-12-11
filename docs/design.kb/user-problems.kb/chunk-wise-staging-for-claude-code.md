# Chunk-wise Staging for Claude Code

Claude Code agents need the ability to commit selective hunks from their changes, equivalent to `git commit -p` or `git add -p`.

## Context

- Multiple agents may work in the same repo simultaneously
- Working copy is always dirty with unrelated changes
- Index may contain staged changes from other work
- Agents need to commit only their specific changes without affecting others

## Requirements

1. Commit specific hunks, not whole files
2. Don't commit unrelated indexed changes
3. Don't revert uncommitted working copy changes
4. Allow human review/editing of hunks before commit
