---
solutions:
  - ../integration-patterns.kb/isolated-index-commit-workflow.md
---

# Path-Qualified Commit Uses Working Copy

`git commit -m "msg" -- paths` commits from the working copy, not the index.

## Verified Behavior

```bash
# Setup: staged change (line1), additional working copy change (line2)
git add file.txt           # stages line1 change
echo "line2" >> file.txt   # additional unstaged change

git commit -m "msg" -- file.txt
# Result: BOTH line1 and line2 committed
```

## Implication

Cannot use path qualification to commit only staged hunks. The index is bypassed entirely for specified paths.
