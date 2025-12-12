# Functional Testing Breakages

Post-hoc TDD tracking for git-commit-staged.

## What belongs here

Each file describes one way to break the implementation. The file exists because:
- We identified a plausible bug
- We want to verify test coverage catches it

## What does NOT belong

- Test documentation (tests are self-documenting)
- Implementation notes (go in design.kb/)
- Bug reports from users (those are issues)

## Lifecycle

1. Create file with `status: todo` when identifying a breakage
2. Attempt the breakage following TDD-posthoc procedure
3. Update to `status: done` when tests catch the bug
