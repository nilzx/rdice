# rdice TUI Design

## Background

`rdice` already separates reusable dice behavior into `rdice-core` and one-shot command usage into `rdice-cli`. The TUI has a different product role: it is a persistent virtual tray workspace for managing dice combinations, inspecting tray state, and rolling one or more trays when needed.

The future GUI can provide richer visual management. The TUI should stay compact, keyboard-friendly, and focused on fast repeated operation.

## Product Scope

The first TUI iteration includes:

- Tray overview dashboard.
- Single tray detail page.
- Custom die manager page.
- Tray manager page.
- Roll history page.
- Command mode entered with `:`.
- Persistent state through the existing `rdice-tui` storage layer.

The first iteration does not include:

- Search or fuzzy filtering.
- Custom die edit forms.
- Tray renaming.
- GUI-shared app crate extraction.
- Complex modal/window stacking beyond simple input flows.

## Page Model

### Overview

Overview is the main workspace. It displays trays as a paged grid rather than a cursor-driven list.

If the loaded TUI state has no trays, the app creates and persists a `default` tray before rendering Overview.

Each tray block shows:

- Stable numeric page-local ID from `[1]` through `[9]`.
- Tray name.
- Selected marker, shown as `*`.
- Dice composition list.
- Numeric total from current rolled values.
- Optional inline range and expected value annotation.
- Hidden text result marker when text results exist but are hidden.

The composition line uses compact notation:

- Same numeric dice are grouped as `[count]x[name]`, for example `2xd6`.
- Single dice are shown as their names, for example `d20`.
- Long custom die names are truncated with `...`.

The total line formats analysis inline:

```text
Total:8
Total(2-24):8
Total(~12.0):8
Total(2-24~12.0):8
Total(2-24~12.0):-
Total(2-24~12.0):8 +t
```

Rules:

- `Total` only sums currently rolled numeric results.
- Unrolled trays display `Total:-`.
- Range and EV can still be shown for unrolled trays.
- EV is rounded to one decimal place.
- Text results are hidden by default.
- `+t` means hidden non-numeric results exist.
- `+t` is right-aligned within the tray block when width allows.

The grid uses tight blocks with no blank lines inside each tray block. Blocks are separated by aligned horizontal and vertical borders. It only renders blocks for trays that exist on the current page: one tray produces one block, two trays produce two blocks, and rows contain at most three blocks. Each page shows at most nine trays so every tray can be addressed with a single numeric key. When trays exceed the visible area or the nine-tray page limit, Overview uses pages rather than scrolling.

Overview keys:

```text
1-9       select or deselect tray on the current page
o<num>    open tray detail
r         roll selected trays; if none selected, no-op with a status message
t         show/hide text results
R         show/hide range annotations
E         show/hide EV annotations
h         open Roll History
m         open Tray Manager
PgUp      previous page
PgDn      next page
:         command mode
q         quit
```

### Tray Detail

Tray Detail shows one tray as a vertical slot list. It avoids cursor movement and uses direct numeric slot operations.

Each slot row shows:

- Slot numeric ID.
- Die name.
- Lock state.
- Current value.

Keys:

```text
r         roll current tray
l<num>    lock/unlock slot
d<num>    remove slot
a         add die to current tray
h         open Roll History
m         open Dice Manager
Esc       return to Overview
:         command mode
```

`l3` toggles lock state for slot 3. `d3` removes slot 3.

### Add Die Flow

From Tray Detail, `a` opens a simple add-die selection view.

Initial scope:

- Show available dice in a paged numbered list.
- `1-9` adds the matching die on the current page.
- `PgUp` and `PgDn` change pages when needed.
- `Esc` returns to the current tray detail page.
- No search/filter in the first iteration.

### Roll History

Roll History shows the most recent tray rolls recorded by the TUI.

Each entry shows:

- Most recent rolls first.
- Tray name.
- Numeric total when one exists.
- Slot values, including lock markers.

The initial history keeps a bounded recent list and is persisted with the TUI
state.

Keys:

```text
Esc       return to the previous page
```

### Dice Manager

Dice Manager is a separate page for custom dice only. It does not manage tray membership.

First iteration capabilities:

- View custom dice.
- Create custom dice through command mode.
- Delete custom dice through command mode.
- Edit custom dice through command mode.
- Use manager shortcuts to prefill commands.

Keys:

```text
n         prefill :dice new
d<num>    prefill :dice delete <name>
e<num>    prefill :dice edit <name>
Esc       return to the previous page
```

Commands:

```text
:manager dice
:dice new <name> <faces...>
:dice delete <name>
:dice edit <name> <faces...>
```

Face parsing follows the CLI/core convention: integer-looking faces become numeric faces; all other faces become text faces.

### Tray Manager

Tray Manager is a separate page for tray lifecycle only. It does not manage individual tray slots.

First iteration capabilities:

- View trays.
- Create trays through command mode.
- Delete trays through command mode.
- Rename trays through command mode.
- Use manager shortcuts to prefill commands.

Keys:

```text
n         prefill :tray new
d<num>    prefill :tray delete <name>
e<num>    prefill :tray rename <old>
Esc       return to Overview
```

Commands:

```text
:manager trays
:tray new <name>
:tray delete <name>
:tray rename <old> <new>
```

## Command Mode

Command mode is entered with `:`. It supports global navigation and manager operations.

The footer is screen-aware. Its left side is reserved for command input, key-prefix feedback such as `open` or `lock/unlock`, and command/action results such as `added D6 to combat`. Its right side shows the current screen's shortcuts. On narrow terminals, feedback takes priority over shortcut help.

Initial commands:

```text
:manager dice
:manager trays
:history
:overview
:tray <name>
:dice new <name> <faces...>
:dice delete <name>
:dice edit <name> <faces...>
:tray new <name>
:tray delete <name>
:tray rename <old> <new>
:quit
```

Command mode errors are shown in the footer feedback area and do not mutate state. Successful mutating commands also report their result there.

## Architecture

The TUI stays inside `crates/rdice-tui`. `rdice-core` remains the business layer for dice definitions, tray state, rolling, locking, and analysis.

Proposed module layout:

```text
crates/rdice-tui/src/
  main.rs
  app.rs
  command.rs
  input.rs
  ui.rs
  screens/
    overview.rs
    tray.rs
    dice_manager.rs
    tray_manager.rs
  storage.rs
```

Responsibilities:

- `main.rs`: terminal setup, cleanup, application bootstrap.
- `app.rs`: global state, screen transitions, mutation methods.
- `command.rs`: command parsing and command enum.
- `input.rs`: key handling and input sequence handling.
- `ui.rs`: root rendering composition.
- `screens/*`: screen-specific rendering and small view helpers.
- `storage.rs`: existing save/load contract.

The app state should contain:

```text
engine: DiceEngine
screen: Screen
selected_trays: set/list of tray names
page: usize
command_mode: inactive or editing buffer
overview_text_visible: bool
overview_range_visible: bool
overview_ev_visible: bool
message: optional status/error text
should_quit: bool
```

Input and command parsing should produce app-level actions. They should not directly mutate `DiceEngine`. Mutations should happen through explicit `App` methods such as:

```text
roll_selected_trays()
open_tray(name)
toggle_tray_selection(page_id)
toggle_slot_lock(slot_id)
remove_slot(slot_id)
create_custom_die(...)
delete_custom_die(...)
create_tray(...)
delete_tray(...)
```

This keeps terminal input concerns separate from application behavior and leaves a cleaner path for future GUI ideas.

## Persistence

The existing `storage::load` and `storage::save` behavior remains the persistence contract.

Successful mutating actions save immediately:

- Roll tray or trays.
- Add die to tray.
- Remove slot.
- Lock/unlock slot.
- Create/delete custom die.
- Create/delete tray.

Failed actions do not save. They show an error message in the status bar.

## Error Handling

The TUI should expose core errors directly as user-visible messages. It should not silently recover from invalid operations.

Examples:

- Unknown die.
- Unknown tray.
- Invalid command.
- Invalid slot ID.
- Cannot delete custom die that is still used by trays.

## Rendering Constraints

The Overview grid must be responsive to terminal size:

- Compute block width and number of columns from available area.
- Keep tray blocks aligned with a single border between adjacent cells.
- Prefer paging over vertical scrolling.
- Truncate long names and lines deterministically.
- Avoid Unicode symbols for essential state when ASCII is sufficient.

`+t`, `*`, `R`, and `E` are ASCII-friendly status markers.

## Testing Strategy

Core behavior should remain covered in `rdice-core` tests.

TUI tests should focus on:

- Command parsing.
- App state transitions.
- Overview tray summarization.
- Total/range/EV formatting.
- Text hiding marker behavior.
- Slot operation parsing, such as `l3` and `d3`.
- Storage round-trip behavior, reusing the existing storage tests.

Terminal rendering can be tested through focused formatter/view-model functions instead of brittle full-screen snapshots in the first iteration.
