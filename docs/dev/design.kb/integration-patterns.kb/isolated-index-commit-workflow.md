# Isolated Index Commit Workflow

Commit specific hunks without affecting the main index or working copy.

## Mechanism

Use `GIT_INDEX_FILE` environment variable to operate on a separate index.

## Workflow

```bash
# 1. Create isolated index from HEAD
GIT_INDEX_FILE="$PWD/.git/index.partial-$UUID" git read-tree HEAD

# 2. Apply desired hunks to isolated index
GIT_INDEX_FILE="..." git apply --cached selected.patch

# 3. Review staged changes
GIT_INDEX_FILE="..." git diff --cached

# 4. Commit from isolated index (plumbing)
TREE=$(GIT_INDEX_FILE="..." git write-tree)
COMMIT=$(git commit-tree "$TREE" -p HEAD -m "message")
git update-ref HEAD "$COMMIT"

# 5. Cleanup
rm "$GIT_INDEX_FILE"
```

## Properties

- Main index untouched (other agents' staged work preserved)
- Working copy untouched (uncommitted changes remain)
- Hunk-level granularity via patch application
- Human can edit patches before application
