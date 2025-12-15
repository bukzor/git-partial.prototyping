# Two-tier integration testing: CLI vs git-integration

**Date:** 2025-12-15
**Status:** Accepted

## Context

Mutation testing revealed gaps where tests passed even with bugs injected. The root cause: CLI integration tests (both git and git-commit-staged as subprocesses) can only inspect exit codes and stderr strings. When a mutation changes the error path but still produces a failure, string matching either:
- Accepts any failure (too loose, mutation survives)
- Requires exact string match (brittle, breaks on message changes)

Example: The `ignore-index-conflict-stages` mutation skips `Delta::Conflicted` entries. Without mutation: error is `git2::ErrorClass::Index` ("invalid entry mode"). With mutation: error is "no staged changes". Both are failures, but only one is correct behavior.

## Decision

Two tiers of integration tests:

1. **CLI integration** (`tests/integration.rs`): Both git and git-commit-staged run as subprocesses. Tests user-facing behavior: exit codes, stdout/stderr content, resulting git state.

2. **Git integration** (`tests/git_integration.rs`): Git runs as subprocess, git-commit-staged runs in-process via library function. Tests can inspect Rust types: `Result`, `anyhow::Error` chains, `git2::Error` class/code.

To enable git-integration tests, extract core logic into `src/lib.rs` with a `git_commit_staged()` function that takes plain values and returns `Result<CommitResult>`.

## Alternatives Considered

### Single tier: CLI-only tests with exact string matching
- **Pros:** Simpler, one test file, tests exactly what users see
- **Cons:** Brittle (message changes break tests), can't distinguish error types, mutations survive

### Single tier: All in-process
- **Pros:** Full introspection, no subprocess overhead
- **Cons:** Doesn't test CLI argument parsing, exit codes, actual user experience

### Mocking git2
- **Pros:** Fast, deterministic, can simulate any scenario
- **Cons:** Doesn't test real git behavior, mock drift, significant implementation effort

## Consequences

**Positive:**
- Mutation testing can verify error types, not just "did it fail"
- Tests are more precise about what they verify
- Library extraction enables future reuse

**Negative:**
- Two test files to maintain
- Must decide which tier each test belongs to
- Library API becomes public contract

**Neutral:**
- Slightly more code (lib.rs + main.rs vs single main.rs)
- Test categorization becomes explicit

## Related

- Related to: `docs/dev/mutation-testing.kb/` gap closure effort
- Related to: `docs/dev/devlog/2025-12-15-mutation-testing-gap-closure.md`
