---
status: equivalent
---

# Remove Exact Path Match Check

Hypothesis: removing the equality check would break exact file matching.

Result: `Path::starts_with()` is component-aware and subsumes equality. `"a/b".starts_with("a/b")` returns true.

The equality check was redundant. Removed as simplification.
