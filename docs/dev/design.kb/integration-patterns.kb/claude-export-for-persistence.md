# claude-export for Persistence

Persist environment variables across Claude Code Bash() tool invocations.

## Problem

Shell state (variables, PWD changes) doesn't persist between Bash() calls. Each invocation is a fresh shell.

## Solution

Use `claude-export` helper to persist variables to the Claude Code session:

```bash
# Call 1: Set variable
claude-export GIT_PARTIAL_IDX="$PWD/.git/index.partial-$(uuidgen)"

# Call 2: Variable available
echo "$GIT_PARTIAL_IDX"  # Works!
```

## Usage Pattern

```bash
# Initialize once per commit session
claude-export GIT_PARTIAL_IDX="$PWD/.git/index.partial-$(uuidgen)"

# Use in subsequent calls
GIT_INDEX_FILE="$GIT_PARTIAL_IDX" git read-tree HEAD
GIT_INDEX_FILE="$GIT_PARTIAL_IDX" git apply --cached ...
```

## Note

This is specific to Claude Code's environment. A standalone `git-partial` tool would manage its own state via files.
