# rdice-core

`rdice-core` is the reusable domain library behind `rdice`. It provides dice
definitions, custom dice, persistent tray data structures, compact expression
parsing, random rolling, and deterministic range and expected-value analysis.

## Features

- Built-in numeric dice from D4 through D100
- Serializable custom dice and tray types
- Compact expressions such as `3d6`, `d20`, and `2Coin`
- Roll results, integer sums, point ranges, and expected values
- No CLI, TUI, filesystem, or application configuration dependencies

## Example

```rust
use rdice_core::{DiceEngine, FaceValue, parse_roll_exprs};

let mut engine = DiceEngine::new();
engine.create_die(
    "Coin",
    vec![FaceValue::Text("heads".into()), FaceValue::Text("tails".into())],
)?;

let parsed = parse_roll_exprs(&["2d6", "Coin", "3"])?;
let analysis = engine.analyze_roll(&parsed.dice, &parsed.modifiers)?;

assert_eq!(analysis.point_range.min, 5);
assert_eq!(analysis.point_range.max, 15);

# Ok::<(), rdice_core::DiceError>(())
```

## Usage

```toml
[dependencies]
rdice-core = "0.1"
```

The Rust crate name is `rdice_core`.

## License

Licensed under the MIT License.
