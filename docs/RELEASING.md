# Releasing

This document describes the release flow for the public crates in this
workspace.

## Prerequisites

1. Create a crates.io account.
2. Create a crates.io API token.
3. Log in locally:

```sh
cargo login
```

## Before Publishing

Run the full verification suite:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Check the package contents:

```sh
cargo package -p rdice-core --list
cargo package -p rdice-cli --list
cargo package -p rdice-tui --list
```

Before `rdice-core` has been published to crates.io, `cargo package -p rdice-cli`
and `cargo package -p rdice-tui` may fail during verification because Cargo
checks registry dependencies for packages being prepared for upload. Use
`--no-verify` only to inspect package contents before the dependency exists in
the registry:

```sh
cargo package -p rdice-cli --no-verify --list
cargo package -p rdice-tui --no-verify --list
```

Do not use `--no-verify` for the final publish flow.

## Publish Order

Publish dependency roots first:

```sh
cargo publish -p rdice-core --dry-run
cargo publish -p rdice-core

cargo publish -p rdice-cli --dry-run
cargo publish -p rdice-cli

cargo publish -p rdice-tui --dry-run
cargo publish -p rdice-tui
```

Wait for crates.io to index `rdice-core` before publishing packages that depend
on it.

## Version Updates

Update the version in the package being released. If `rdice-core` changes, also
update the workspace dependency requirement used by downstream packages:

```toml
rdice-core = { version = "0.1.0", path = "crates/rdice-core" }
```

Published versions cannot be overwritten. If a release is broken, publish a new
version. Use `cargo yank` only to prevent new dependency resolution to a bad
version:

```sh
cargo yank --version 0.1.0 rdice-core
```

## Tags

Use package-scoped tags:

```text
rdice-core-v0.1.0
rdice-cli-v0.1.0
rdice-tui-v0.1.0
```
