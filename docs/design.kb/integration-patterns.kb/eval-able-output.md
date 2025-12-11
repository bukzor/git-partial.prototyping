# Eval-able Output Pattern

Tool outputs environment variable assignments that the caller evaluates.

## Pattern

Like `ssh-agent`, output statements the shell can eval:

```bash
# Tool outputs assignment
$ git partial init
GIT_PARTIAL_SESSION=/path/to/.git/partial-sessions/abc123

# Caller evals in their context
$ eval "$(git partial init)"
```

## Output Formats

```bash
git partial init --sh     # VAR=value; export VAR
git partial init --fish   # set -x VAR value
git partial init --json   # {"VAR": "value"}
```

## Benefits

- Tool doesn't need to know caller's environment
- Caller decides how to persist (eval, claude-export, etc.)
- Composable with other tools
- Testable - output is inspectable text

## Claude Code Usage

```bash
# Persist across Bash() calls
git partial init --sh | xargs -L1 claude-export

# Or single-var shorthand
claude-export "$(git partial init)"
```
