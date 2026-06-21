# rdice-tui

`rdice-tui` is a terminal UI for persistent dice trays. It supports repeated
tray rolling, slot locking, custom dice management, and compact keyboard-driven
operation.

## Install

```sh
cargo install rdice-tui
```

## Usage

```sh
rdice-tui
```

The state path can be overridden with `RDICE_TUI_STATE_PATH`; otherwise it uses:

```text
~/.local/state/rdice-tui/state.toml
```

## Keys

Overview:

```text
1-9       select/deselect tray on the current page
o<num>    open tray detail
r         roll selected trays
t         show/hide text results
R         show/hide range annotations
E         show/hide expected value annotations
h         show roll history
m         open tray manager
PgUp/PgDn change page
:         command mode
q         quit
```

Tray detail:

```text
r         roll current tray
l<num>    lock/unlock slot
d<num>    remove slot
a         add die
h         show roll history
m         open custom dice manager
Esc       return to overview
:         command mode
```

Commands:

```text
:manager dice
:manager trays
:history
:dice new fate -1 0 +
:dice delete fate
:dice edit fate -1 0 1
:tray new combat
:tray delete combat
:tray rename combat battle
```

## License

Licensed under the MIT License.
