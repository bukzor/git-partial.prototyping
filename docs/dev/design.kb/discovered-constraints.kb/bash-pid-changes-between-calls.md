---
solutions:
  - ../integration-patterns.kb/claude-export-for-persistence.md
---

# Bash PID Changes Between Calls

`$$` (shell PID) changes between Claude Code Bash() tool invocations.

## Problem

Cannot use `$$` to generate stable temporary file paths across multiple tool calls:

```bash
# Call 1
GIT_INDEX_FILE="/tmp/index.$$"  # e.g., /tmp/index.12345

# Call 2
echo $GIT_INDEX_FILE  # Different PID, variable not set
```

## Implication

Need explicit persistence mechanism for state that spans Bash() calls.
