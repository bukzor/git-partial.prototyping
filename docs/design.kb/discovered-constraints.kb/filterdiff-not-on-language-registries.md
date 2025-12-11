---
solutions:
  - ../technology-choices.kb/diff-parsing-library.md
---

# filterdiff Not on Language Registries

The `patchutils` package (containing `filterdiff`, `lsdiff`, `splitdiff`) is only available via system package managers.

## Available Via

- Homebrew: `brew install patchutils`
- Debian/Ubuntu: `apt install patchutils`
- Fedora: `dnf install patchutils`
- Arch: `pacman -S patchutils`

## NOT Available Via

- PyPI (different unrelated package named "patchutils")
- crates.io (different unrelated package)
- npm

## Implication

A tool depending on `filterdiff` cannot express this dependency in Cargo.toml, pyproject.toml, or package.json. Must use native language libraries instead for portable distribution.
