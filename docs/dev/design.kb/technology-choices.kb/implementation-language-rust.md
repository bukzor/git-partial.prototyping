# Implementation Language: Rust

## Decision

Implement git-partial in Rust.

## Alternatives Considered

| Language | Pros | Cons |
|----------|------|------|
| **Rust** | User's learning goal, git-absorb as model, `git2` crate | Steeper learning curve |
| Python | `unidiff` library excellent, quick iteration | User tired of typing fights |
| TypeScript | `parse-diff` available | Manual patch serialization |

## Rationale

- User has one-year Rust expertise goal
- `git-absorb` provides proven architecture to emulate
- `git2` crate for libgit2 bindings
- `patch` crate (484k downloads) for diff parsing
- Single binary distribution

## Model: git-absorb

- Small codebase (336KB)
- Minimal deps: clap, git2, anyhow
- Uses git2 for repo operations (not shelling out)
- Packaged across many distros
