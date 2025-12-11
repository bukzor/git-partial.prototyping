# Hunk File Selection

User selects hunks by manipulating files in a directory structure.

## Structure

```
.git/hunks.d/
  {filename}/
    {start}-{end}.patch
```

Example:
```
.git/hunks.d/
  src/main.rs/
    1-5.patch
    45-60.patch
  README.md/
    10-15.patch
```

## Line Numbers

Use left-hand (old/source) line numbers from the `@@` header:
- Stable reference to HEAD
- Doesn't shift as hunks are applied
- User can `git show HEAD:file | sed -n 'start,endp'` for context

## Selection Operations

- **Include**: keep the .patch file
- **Exclude**: `rm` the .patch file
- **Edit**: modify the .patch file with text editor

## Properties

- Human can review/edit hunks out-of-band
- Clear mapping from filename to affected code region
- `ls` shows available hunks
- Standard Unix file operations for selection
