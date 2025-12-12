# Path-Scoped Index Commit Workflow

Commit staged changes at specific paths without affecting other staged work.

## Mechanism

Use `GIT_INDEX_FILE` to build isolated index containing only specified paths from main index.

## Workflow

```bash
PATHS="src/foo.rs tests/test_foo.rs"
ISO_IDX="$PWD/.git/index.commit-staged-$$"

# 1. Create isolated index from HEAD (baseline)
GIT_INDEX_FILE="$ISO_IDX" git read-tree HEAD

# 2. For each path, copy staged entry from main index to isolated index
for path in $PATHS; do
  # Get blob hash from main index
  BLOB=$(git ls-files --stage -- "$path" | awk '{print $2}')
  MODE=$(git ls-files --stage -- "$path" | awk '{print $1}')

  # Update isolated index with this entry
  GIT_INDEX_FILE="$ISO_IDX" git update-index --cacheinfo "$MODE,$BLOB,$path"
done

# 3. Commit from isolated index (plumbing)
TREE=$(GIT_INDEX_FILE="$ISO_IDX" git write-tree)
COMMIT=$(git commit-tree "$TREE" -p HEAD -m "message")
git update-ref HEAD "$COMMIT"

# 4. Remove committed paths from main index (reset to new HEAD)
git reset HEAD -- $PATHS

# 5. Cleanup
rm "$ISO_IDX"
```

## Properties

- Only specified paths committed
- Other staged changes preserved in main index
- Working copy untouched
- Main index updated only to reflect committed paths
