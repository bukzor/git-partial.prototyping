---
status: done
---

# Deletions Become Modifications

If delta status check is wrong, staged deletions could be treated as modifications, causing the commit to fail when trying to read blob content that doesn't exist.

Inject by changing `Delta::Deleted` match to something else.
