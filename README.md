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

### git-commit-files

Stage and commit working tree changes at specific paths in one step.

```bash
git commit-files src/ -- -m "Add feature"
```

Equivalent to `git add src/ && git commit-staged src/ -- -m "..."` but atomic.

## Installation

**Homebrew (recommended):**

The formula lives in this repo, so tap it directly:

```bash
brew tap bukzor/git-partial https://github.com/bukzor/git-partial.prototyping
brew install bukzor/git-partial/git-commit-staged
```

**Cargo (manual man page install):**

```bash
cargo install --locked --path git-commit-staged
ln -s "$PWD/git-commit-staged/man"/*.1 ~/.local/share/man/man1/
```

On musl targets (e.g. Alpine), Rust links statically by default and fails
with `cannot find -lssl` / `-lcrypto` / `-lz`. Either link dynamically
against the system libraries:

```bash
RUSTFLAGS='-C target-feature=-crt-static' cargo install --locked --path git-commit-staged
```

or install the static libraries (`apk add openssl-libs-static zlib-static`).

## Usage

Arguments after `--` pass through to `git commit`:

```bash
git commit-staged src/ tests/ -- --amend
git commit-files . -- --fixup HEAD~1
```

Dry run to see what would be committed:

```bash
git commit-staged -n src/
git commit-files -n src/
```

## Status

Prototyping phase. See `docs/dev/` for design documentation.
