# git-partial

Chunk-wise staging for Claude Code agents, emulating `git commit -p` non-interactively.

## Commands

### git-commit-staged

Commit staged changes at specific paths only, using the index (not the working copy).

```bash
git add src/
git commit-staged src/ -- -m "Add feature"
```

Unlike `git commit -- paths`, this commits from the index, not the working copy.

### git-commit-working

Stage and commit working tree changes at specific paths in one step.

```bash
git commit-working src/ -- -m "Add feature"
```

Equivalent to `git add src/ && git commit-staged src/ -- -m "..."` but atomic.

## Installation

**Homebrew (recommended):**

```bash
brew install bukzor/tap/git-commit-staged
```

**Cargo (manual man page install):**

```bash
cargo install --path git-commit-staged
ln -s "$PWD/git-commit-staged/man"/*.1 ~/.local/share/man/man1/
```

## Usage

Arguments after `--` pass through to `git commit`:

```bash
git commit-staged src/ tests/ -- --amend
git commit-working . -- --fixup HEAD~1
```

Dry run to see what would be committed:

```bash
git commit-staged -n src/
git commit-working -n src/
```

## Status

Prototyping phase. See `docs/dev/` for design documentation.
