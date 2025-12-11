# Git Subcommand Discovery

How git finds and runs external subcommands.

## Mechanism

Any executable named `git-{name}` on PATH becomes `git {name}`.

```bash
# Put git-partial on PATH
cargo install git-partial

# Now works automatically
git partial init
git partial commit
```

## What Git Provides

- `git {name}` → finds and runs `git-{name}`
- `git help -a` → lists it under "External commands"
- `git {name} --help` → looks for `man git-{name}`

## Environment Passed to Subcommand

```
GIT_EXEC_PATH=/path/to/git-core
PATH=<git-core prepended>:<original PATH>
PWD=<user's cwd>
```

## No Registration Required

Unlike plugins in other systems, git subcommands need no configuration. Just put the executable on PATH.
