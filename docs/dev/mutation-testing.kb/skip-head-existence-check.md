---
status: done
---

# Empty Repo No HEAD

On a fresh repo with no commits, `repo.head()` fails or returns an unborn branch.

## Injection
Change `find_staged_entries` to return empty Vec instead of erroring when HEAD doesn't exist.

## Test Coverage
`errors_on_empty_repo` test catches this - it verifies the error message mentions HEAD. Mutation causes tool to silently report "no staged changes" which fails the assertion.
