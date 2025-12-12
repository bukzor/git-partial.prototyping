---
status: done
---

# Partial Path False Positive

`path_matches` uses `starts_with` which should handle path components correctly (not string prefix). But if implemented as string comparison, "src/foo" would incorrectly match "src/foobar/baz.rs".

There's already a unit test for this (`no_false_prefix_match`). Verify the integration test also catches it.
