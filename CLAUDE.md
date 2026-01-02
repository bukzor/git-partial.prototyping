---
depends:
  - skills/llm.kb
  - skills/llm-collab
---

# git-partial

Chunk-wise staging for Claude Code agents, emulating `git commit -p` non-interactively.

## Design Knowledge

See `docs/dev/design.kb/` for architectural decisions, constraints, and patterns.

## Status

Prototyping phase. Core functionality implemented.

## After Pushing

Reinstall to update the binaries:

```bash
cargo install --path git-commit-staged
```
