---
status: done
---

# Mode Changes Ignored

If we only compare blob OIDs and ignore filemode, a `chmod +x` that's staged would not be detected as a change.

Inject by checking only `id` equality, ignoring mode differences in the diff.
