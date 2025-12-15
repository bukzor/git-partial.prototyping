# git-partial

Chunk-wise staging for Claude Code agents, emulating `git commit -p` non-interactively.

## git-commit-staged

Commit staged changes at specific paths only, using the index (not the working copy).

### Installation

**Homebrew (recommended):**

```bash
brew install bukzor/tap/git-commit-staged
```

**Cargo (manual man page install):**

```bash
cargo install --path git-commit-staged
cp git-commit-staged/man/git-commit-staged.1 /usr/local/share/man/man1/
```

### Usage

```bash
git add src/
git commit-staged src/ -- -m "Add feature"
```

Unlike `git commit -- paths`, this commits from the index, not the working copy.

Arguments after `--` pass through to `git commit`:

```bash
git commit-staged src/ tests/ -- --amend
git commit-staged . -- --fixup HEAD~1
```

Dry run to see what would be committed:

```bash
git commit-staged -n src/
```

## Status

Prototyping phase. See `docs/dev/` for design documentation.
