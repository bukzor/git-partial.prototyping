---
status: done
---

# Skip Deleted Deltas

If `find_staged_entries` skips `Delta::Deleted` entries entirely, staged deletions (`git rm`) would not be committed.

Inject by changing the Deleted match arm to `continue` instead of returning `(path_str, None)`.
