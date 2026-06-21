# rdice

English | [中文](README_zh.md)

`rdice` is a Rust dice project organized as a public workspace for reusable
dice logic, command-line tools, and terminal UI tools.

## Projects

| Project | Type | Package | Output | Publish target |
| --- | --- | --- | --- | --- |
| `rdice-core` | Library | `rdice-core` | Rust crate `rdice_core` | crates.io |
| `rdice-cli` | CLI | `rdice-cli` | `rdice` binary | crates.io |
| `rdice-tui` | TUI | `rdice-tui` | `rdice-tui` binary | crates.io |

Private application projects such as `rdice-app` and `rdice-web` should live in
separate private repositories and consume the published `rdice-core` crate.

## Repository Layout

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README_zh.md
├── crates/
│   ├── rdice-core/
│   ├── rdice-cli/
│   └── rdice-tui/
└── docs/
    ├── ARCHITECTURE.md
    ├── RELEASING.md
    ├── plan/
    └── spec/
```

## Install

Install the command-line dice roller:

```sh
cargo install rdice-cli
rdice roll 3d6
```

Install the terminal UI:

```sh
cargo install rdice-tui
rdice-tui
```

Use the core library from another Rust project:

```toml
[dependencies]
rdice-core = "0.1"
```

```rust
use rdice_core::{DiceEngine, parse_roll_exprs};

fn main() -> Result<(), rdice_core::DiceError> {
    let engine = DiceEngine::new();
    let parsed = parse_roll_exprs(&["2d6", "3"])?;
    let analysis = engine.analyze_roll(&parsed.dice, &parsed.modifiers)?;

    assert_eq!(analysis.point_range.min, 5);
    Ok(())
}
```

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## CLI

```sh
rdice roll [-f|--folded] [-x|--expanded] [-E|--ev] [-R|--range] <dice-expr...>
rdice [-E|--ev] [-R|--range] <dice-expr...>
rdice list
rdice config path
rdice config edit
rdice help [--no-color]
```

Examples:

```sh
rdice roll 5d6
rdice roll -x 4d6
rdice roll -f -E -R 3d13 2coin 5 -3
rdice -E -R 3d6 5 -3
```

CLI output uses ANSI colors by default. Pass `--no-color` or set `NO_COLOR` to
disable color output for scripts and plain-text logs. The TUI also respects
`NO_COLOR`.

Custom dice are loaded from `RDICE_CONFIG_PATH` when set, otherwise from:

```text
~/.config/rdice/config.toml
```

Example config:

```toml
[[dice]]
name = "coin"
faces = ["heads", "tails"]

[[dice]]
name = "fate"
faces = [-1, 0, 1]
```

## TUI

`rdice-tui` is an interactive virtual tray workspace for persistent dice trays,
repeated tray rolling, slot locking, and compact keyboard-driven operation.

The TUI state path can be overridden with `RDICE_TUI_STATE_PATH`; otherwise it
uses:

```text
~/.local/state/rdice-tui/state.toml
```

Start it with:

```sh
rdice-tui
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Release process](docs/RELEASING.md)
- [TUI design spec](docs/spec/2026-04-27-tui-design.md)
- [TUI implementation plan](docs/plan/2026-04-27-tui-implementation-plan.md)

## License

Licensed under the [MIT License](LICENSE).
