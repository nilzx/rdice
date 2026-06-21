# Contributing

Thanks for contributing to `rdice`.

## Development

Run the standard checks before opening a pull request:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Workspace Rules

- Keep reusable dice logic in `rdice-core`.
- Keep CLI-specific config, output, and process behavior in `rdice-cli`.
- Keep terminal UI state, rendering, and input handling in `rdice-tui`.
- Do not add private app or web code to this public repository.

## Public API Changes

Changes to `rdice-core` can affect every downstream project. When changing its
public API, update affected tests and downstream workspace crates in the same
pull request.

## Releases

See [docs/RELEASING.md](docs/RELEASING.md).
