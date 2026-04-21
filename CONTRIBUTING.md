# Contributing

This project does not have an open-source license yet. Please wait for a
license before relying on broad reuse rights.

## Local Setup

Install Rust with `rustup`, then run:

```sh
cargo build
cargo run
```

Run the TUI in a real terminal. Some terminal behavior will not work correctly
inside non-interactive automation.

## Checks

Before opening a pull request, run:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```

## Pull Requests

- Keep changes scoped to one behavior or fix.
- Do not commit local vaults, wallet exports, logs, `.env` files, API keys,
  private keys, seed phrases, or screenshots containing sensitive data.
- Add or update tests for behavior changes.
- Document user-visible changes in `README.md` when relevant.
