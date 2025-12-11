# Subsystem Breakdown

## Decision

Split git-partial into focused internal crates with clear responsibilities.

## Subsystems

| Crate | Responsibility |
|-------|----------------|
| **patch** | Parse unified diff → hunks, serialize hunks → patches (wraps `patch` crate) |
| **git** | Plumbing ops: read-tree, apply --cached, write-tree, commit-tree, update-ref |
| **session** | Session lifecycle: create, load, cleanup; owns the session directory |
| **hunks** | Hunk file management: split diff → files, enumerate, apply selected |
| **export** | Format env assignments: sh, zsh, fish, json |
| **cli** | Subcommands (init, list, drop, commit, abort), arg parsing (clap) |

## Dependency Flow

```
cli
 ├── session
 │    ├── hunks
 │    │    └── patch
 │    └── git
 └── export
```

## Rationale

- Clear API boundaries enforced by crate separation
- Each subsystem testable in isolation
- `publish = false` keeps them internal
- Dependency flow prevents cycles
