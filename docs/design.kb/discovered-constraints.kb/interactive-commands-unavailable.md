---
solutions:
  - ../integration-patterns.kb/isolated-index-commit-workflow.md
  - ../integration-patterns.kb/hunk-file-selection.md
---

# Interactive Commands Unavailable

Claude Code's Bash tool cannot handle interactive commands that require user input.

## Affected Commands

- `git add -p` (interactive hunk selection)
- `git commit -p` (interactive hunk selection)
- `git add -i` (interactive mode)
- `git rebase -i` (interactive rebase)

## Implication

Any solution must use non-interactive alternatives or pre-scripted input.
