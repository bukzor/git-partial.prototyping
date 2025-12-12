# Contributing

## Setup

```bash
cargo build
```

## Running Tests

```bash
cargo test                    # all tests
cargo test path_matching      # unit tests only (by module name)
cargo test --test integration # integration tests only
cargo test <name>             # by test name substring
```

## Linting

Workspace is configured with `clippy::pedantic` and `clippy::nursery` by default.

```bash
cargo clippy                  # lint
cargo fmt                     # format
cargo clippy --all-targets    # include tests
```
