# Nested Workspace Layout

## Decision

Internal crates nested inside their parent crate directory, not flat `crates/`.

## Layout

```
git-partial.prototyping/
├── Cargo.toml              # [workspace]
├── git-partial/
│   ├── Cargo.toml          # publish = true
│   ├── src/
│   ├── cli/                # publish = false
│   ├── export/             # publish = false
│   ├── git/                # publish = false
│   ├── hunks/              # publish = false
│   ├── patch/              # publish = false
│   └── session/            # publish = false
└── claude-export/
    ├── Cargo.toml          # publish = true
    └── src/
```

## Alternatives Considered

| Layout | Pros | Cons |
|--------|------|------|
| **Nested** | Groups related crates visually, clear ownership | Less common, manual setup |
| Flat `crates/` | Conventional, `cargo new` friendly | All crates at same level |
| Single crate + modules | Simplest | No enforced API boundaries |

## Rationale

- Internal crates (`patch`, `git`, etc.) are implementation details of `git-partial`
- Nesting shows this relationship in filesystem structure
- `publish = false` keeps them private; users only see `git-partial` and `claude-export`
- Cargo supports it fine via explicit `[workspace] members`

## Trade-offs Accepted

- Manual directory creation (no `cargo new` shortcut)
- Slightly longer paths in workspace members list
- Can refactor to flat later if annoying
