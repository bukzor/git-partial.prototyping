---
status: gap
attempts: 2
---

# Index Conflicts Unhandled

During a merge conflict, the index has entries at stages 1, 2, 3 (base, ours, theirs) instead of stage 0.

## Injection
Add `if delta.status() == git2::Delta::Conflicted { continue; }` to skip conflicts.

## Attempts
1. Added `errors_on_merge_conflict` test that creates merge conflict
2. Injected skip-Conflicted mutation - test still passed

Tool already fails on conflicts with "invalid entry mode" (in create_commit), not in find_staged_entries. The mutation is unkillable because the error path differs from expected.

## Test Added
`errors_on_merge_conflict` verifies tool fails on merge conflicts (even if error message is poor).
