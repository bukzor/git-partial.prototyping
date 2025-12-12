# Diff Parsing Library

## Decision

Use native language library for diff parsing instead of `filterdiff`.

## Alternatives Considered

| Approach | Pros | Cons |
|----------|------|------|
| **Native library** | Portable, expressible in Cargo.toml | Must evaluate options |
| filterdiff | Proven, canonical | System dependency, not on crates.io |
| Custom awk parser | No dependencies | Fragile, maintenance burden |

## Language-Specific Options

### Rust (chosen)
- `patch` crate (484k downloads) - parses unified diff, has Display for Hunk

### Python (reference)
- `unidiff` - excellent, parses to PatchSet > PatchedFile > Hunk, serializes back

### TypeScript (reference)
- `parse-diff` - parses to files > chunks, manual serialization

## Rationale

Native library allows `cargo install` without system dependencies. The `patch` crate is mature and widely used.
