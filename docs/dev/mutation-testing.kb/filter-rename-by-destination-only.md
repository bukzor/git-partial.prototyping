---
status: gap
---

# Partial Rename Across Scopes

Rename `src/a.rs` -> `tests/b.rs`, then commit only `src/`.

Expected: `src/a.rs` is deleted (the source half of the rename).

Current behavior with rename detection off: probably works (Delete delta for src/a.rs, Add delta for tests/b.rs filtered out).

But worth verifying: does the delete delta for the source have its path in old_file or new_file? If it's in new_file (the destination), the path filter would exclude it and the source wouldn't be deleted.
