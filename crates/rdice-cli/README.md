# rdice-cli

`rdice-cli` provides the `rdice` command-line dice roller. It uses
`rdice-core` for dice definitions, expression parsing, rolling, and
deterministic analysis.

## Install

```sh
cargo install rdice-cli
```

## Usage

```sh
rdice roll [-f|--folded] [-x|--expanded] [-E|--ev] [-R|--range] <dice-expr...>
rdice [-E|--ev] [-R|--range] <dice-expr...>
rdice list
rdice config path
rdice config edit
rdice help
```

Examples:

```sh
rdice roll 5d6
rdice roll -x 4d6
rdice roll -f -E -R 3d13 2coin 5 -3
rdice -E -R 3d6 5 -3
```

## Custom Dice

The CLI reads custom dice from `RDICE_CONFIG_PATH` when set. Otherwise it uses:

```text
~/.config/rdice/config.toml
```

Example:

```toml
[[dice]]
name = "coin"
faces = ["heads", "tails"]

[[dice]]
name = "fate"
faces = [-1, 0, 1]
```

Custom dice can be referenced without the internal custom prefix, so `2coin`
resolves to the configured `coin` die.

## License

Licensed under the MIT License.
