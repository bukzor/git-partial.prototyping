# Environment-Agnostic Design

## Decision

git-partial has no knowledge of Claude Code internals. Session persistence is the caller's responsibility.

## Alternatives Considered

| Approach | Pros | Cons |
|----------|------|------|
| **Environment-agnostic** | Portable, testable, composable | Caller must handle persistence |
| Claude Code integration | Automatic persistence | Tight coupling, not portable |
| claude-session sub-crate | Reusable detection | Over-engineering, still coupled |

## Rationale

- Unix philosophy: do one thing, output text
- Works in any shell environment, not just Claude Code
- Testable without Claude Code running
- Caller has flexibility (eval, claude-export, source, etc.)

## Implementation

1. `git partial init` creates session, outputs `VAR=value`
2. Subsequent commands read `$GIT_PARTIAL_SESSION` env var
3. Tool doesn't know or care how the var was set
