---
status: gap
---

# Typechange File to Symlink

Staging a change from regular file to symlink (or vice versa) produces `Delta::Typechange`.

Current code hits the `_` arm and uses `new_file()` oid/mode. This might work, but the mode would be different (symlink mode vs regular file mode).

Verify that filemode is correctly preserved through the commit.
