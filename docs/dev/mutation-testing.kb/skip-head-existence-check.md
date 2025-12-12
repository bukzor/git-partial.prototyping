---
status: gap
---

# Empty Repo No HEAD

On a fresh repo with no commits, `repo.head()` fails or returns an unborn branch.

Current code would error at "failed to get HEAD" which is acceptable, but the error message could be clearer: "cannot commit-staged in repo with no commits".

Not strictly a breakage, but worth testing the error path.
