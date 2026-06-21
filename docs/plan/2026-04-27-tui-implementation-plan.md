# rdice TUI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first usable `rdice-tui` virtual tray workspace with paged overview grid, tray detail operations, managers, command mode, and persistent state.

**Architecture:** Keep dice/tray behavior in `rdice-core`; keep terminal state, input, rendering, and persistence orchestration inside `crates/rdice-tui`. Use pure view-model and parser functions for most tests, and keep terminal rendering thin.

**Tech Stack:** Rust 2024, `rdice-core`, `ratatui`, `crossterm`, existing TOML storage.

---

## Notes

- This directory is not currently a git repository, so commit steps are intentionally omitted.
- Follow the spec at `docs/spec/2026-04-27-tui-design.md`.
- Use `cargo fmt` and `cargo test` after each task.
- Keep comments and API docs in English.

## File Structure

Create or modify these files:

- Modify `crates/rdice-tui/Cargo.toml`: add terminal UI dependencies.
- Modify `crates/rdice-tui/src/main.rs`: terminal setup, event loop, app bootstrap.
- Create `crates/rdice-tui/src/app.rs`: app state, screens, persistence-aware actions.
- Create `crates/rdice-tui/src/command.rs`: `:` command parser.
- Create `crates/rdice-tui/src/input.rs`: key/input sequence handling.
- Create `crates/rdice-tui/src/ui.rs`: root ratatui render function.
- Create `crates/rdice-tui/src/screens/mod.rs`: screen modules.
- Create `crates/rdice-tui/src/screens/overview.rs`: overview view model and rendering.
- Create `crates/rdice-tui/src/screens/tray.rs`: tray detail view model and rendering.
- Create `crates/rdice-tui/src/screens/dice_manager.rs`: dice manager view model and rendering.
- Create `crates/rdice-tui/src/screens/tray_manager.rs`: tray manager view model and rendering.
- Keep `crates/rdice-tui/src/storage.rs`: no contract change unless tests expose a narrow need.
- Create `crates/rdice-tui/tests/command_test.rs`.
- Create `crates/rdice-tui/tests/app_test.rs`.
- Create `crates/rdice-tui/tests/overview_test.rs`.
- Create `crates/rdice-tui/tests/input_test.rs`.

---

### Task 1: Add TUI Dependencies And Library Boundary

**Files:**
- Modify `crates/rdice-tui/Cargo.toml`
- Create `crates/rdice-tui/src/lib.rs`
- Modify `crates/rdice-tui/src/main.rs`

- [ ] **Step 1: Add dependencies**

In `crates/rdice-tui/Cargo.toml`, add:

```toml
crossterm = "0.29"
ratatui = "0.30"
```

Keep existing dependencies.

- [ ] **Step 2: Create the library module boundary**

Create `crates/rdice-tui/src/lib.rs`:

```rust
pub mod app;
pub mod command;
pub mod input;
pub mod screens;
pub mod storage;
pub mod ui;
```

- [ ] **Step 3: Replace the skeleton `main.rs` with a thin bootstrap**

Modify `crates/rdice-tui/src/main.rs` so it imports the library modules and still compiles before the full loop is implemented:

```rust
use rdice_core::error::Result;
use rdice_tui::app::App;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut app = App::load_default()?;
    app.save()?;
    println!("rdice-tui app initialized");
    Ok(())
}
```

- [ ] **Step 4: Add a minimal `App` skeleton**

Create `crates/rdice-tui/src/app.rs`:

```rust
use std::path::PathBuf;

use rdice_core::DiceEngine;
use rdice_core::error::{DiceError, Result};

use crate::storage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Overview,
    TrayDetail(String),
    AddDie(String),
    DiceManager,
    TrayManager,
}

#[derive(Debug)]
pub struct App {
    pub engine: DiceEngine,
    pub state_path: PathBuf,
    pub screen: Screen,
    pub selected_trays: Vec<String>,
    pub overview_page: usize,
    pub overview_text_visible: bool,
    pub overview_range_visible: bool,
    pub overview_ev_visible: bool,
    pub command_buffer: Option<String>,
    pub message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn load_default() -> Result<Self> {
        let state_path = default_state_path()?;
        Self::load_from_path(state_path)
    }

    pub fn load_from_path(state_path: PathBuf) -> Result<Self> {
        let engine = match storage::load(&state_path) {
            Ok(engine) => engine,
            Err(err) => {
                eprintln!("Warning: failed to load TUI state: {err}");
                DiceEngine::new()
            }
        };

        Ok(Self {
            engine,
            state_path,
            screen: Screen::Overview,
            selected_trays: Vec::new(),
            overview_page: 0,
            overview_text_visible: false,
            overview_range_visible: false,
            overview_ev_visible: false,
            command_buffer: None,
            message: None,
            should_quit: false,
        })
    }

    pub fn save(&self) -> Result<()> {
        storage::save(&self.state_path, &self.engine)
    }
}

fn default_state_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("RDICE_TUI_STATE_PATH") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir().ok_or_else(|| {
        DiceError::StorageError("cannot determine home directory for TUI state path".to_string())
    })?;

    Ok(home
        .join(".local")
        .join("state")
        .join("rdice-tui")
        .join("state.toml"))
}
```

- [ ] **Step 5: Create empty module files so the crate compiles**

Create these files with minimal content:

```rust
// crates/rdice-tui/src/command.rs
```

```rust
// crates/rdice-tui/src/input.rs
```

```rust
// crates/rdice-tui/src/ui.rs
```

```rust
// crates/rdice-tui/src/screens/mod.rs
pub mod dice_manager;
pub mod overview;
pub mod tray;
pub mod tray_manager;
```

```rust
// crates/rdice-tui/src/screens/overview.rs
```

```rust
// crates/rdice-tui/src/screens/tray.rs
```

```rust
// crates/rdice-tui/src/screens/dice_manager.rs
```

```rust
// crates/rdice-tui/src/screens/tray_manager.rs
```

- [ ] **Step 6: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui
```

Expected: all `rdice-tui` tests pass, including existing storage tests.

---

### Task 2: Command Parser

**Files:**
- Modify `crates/rdice-tui/src/command.rs`
- Create `crates/rdice-tui/tests/command_test.rs`

- [ ] **Step 1: Write command parser tests**

Create `crates/rdice-tui/tests/command_test.rs`:

```rust
use rdice_tui::command::{Command, parse_command};

#[test]
fn parses_navigation_commands() {
    assert_eq!(parse_command("manager dice").unwrap(), Command::ManagerDice);
    assert_eq!(parse_command("manager trays").unwrap(), Command::ManagerTrays);
    assert_eq!(parse_command("overview").unwrap(), Command::Overview);
    assert_eq!(
        parse_command("tray combat").unwrap(),
        Command::OpenTray("combat".into())
    );
    assert_eq!(parse_command("quit").unwrap(), Command::Quit);
}

#[test]
fn parses_dice_commands() {
    assert_eq!(
        parse_command("dice new fate -1 0 +").unwrap(),
        Command::DiceNew {
            name: "fate".into(),
            faces: vec!["-1".into(), "0".into(), "+".into()],
        }
    );
    assert_eq!(
        parse_command("dice delete fate").unwrap(),
        Command::DiceDelete("fate".into())
    );
}

#[test]
fn parses_tray_commands() {
    assert_eq!(
        parse_command("tray new combat").unwrap(),
        Command::TrayNew("combat".into())
    );
    assert_eq!(
        parse_command("tray delete combat").unwrap(),
        Command::TrayDelete("combat".into())
    );
}

#[test]
fn rejects_invalid_commands() {
    assert!(parse_command("").is_err());
    assert!(parse_command("manager").is_err());
    assert!(parse_command("dice new only_name").is_err());
    assert!(parse_command("tray rename a b").is_err());
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p rdice-tui --test command_test
```

Expected: compile failure because `command` types/functions are not implemented.

- [ ] **Step 3: Implement command parser**

Replace `crates/rdice-tui/src/command.rs` with:

```rust
use rdice_core::error::{DiceError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    ManagerDice,
    ManagerTrays,
    Overview,
    OpenTray(String),
    DiceNew { name: String, faces: Vec<String> },
    DiceDelete(String),
    TrayNew(String),
    TrayDelete(String),
    Quit,
}

pub fn parse_command(input: &str) -> Result<Command> {
    let parts = input.split_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return invalid_command(input);
    }

    match parts.as_slice() {
        ["manager", "dice"] => Ok(Command::ManagerDice),
        ["manager", "trays"] => Ok(Command::ManagerTrays),
        ["overview"] => Ok(Command::Overview),
        ["tray", name] => Ok(Command::OpenTray((*name).to_string())),
        ["dice", "new", name, faces @ ..] if !faces.is_empty() => Ok(Command::DiceNew {
            name: (*name).to_string(),
            faces: faces.iter().map(|face| (*face).to_string()).collect(),
        }),
        ["dice", "delete", name] => Ok(Command::DiceDelete((*name).to_string())),
        ["tray", "new", name] => Ok(Command::TrayNew((*name).to_string())),
        ["tray", "delete", name] => Ok(Command::TrayDelete((*name).to_string())),
        ["quit"] | ["q"] => Ok(Command::Quit),
        _ => invalid_command(input),
    }
}

fn invalid_command<T>(input: &str) -> Result<T> {
    Err(DiceError::StorageError(format!("invalid command: {input}")))
}
```

- [ ] **Step 4: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui --test command_test
```

Expected: command tests pass.

---

### Task 3: App Actions And Persistence

**Files:**
- Modify `crates/rdice-tui/src/app.rs`
- Create `crates/rdice-tui/tests/app_test.rs`

- [ ] **Step 1: Write app action tests**

Create `crates/rdice-tui/tests/app_test.rs`:

```rust
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_tui::app::{App, Screen};
use rdice_tui::command::Command;

fn unique_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rdice-tui-app-{label}-{stamp}.toml"))
}

#[test]
fn app_creates_tray_and_persists_it() {
    let path = unique_path("tray");
    let mut app = App::load_from_path(path.clone()).unwrap();

    app.apply_command(Command::TrayNew("combat".into())).unwrap();

    assert!(app.engine.get_tray("combat").is_some());
    assert!(path.exists());

    let loaded = App::load_from_path(path.clone()).unwrap();
    assert!(loaded.engine.get_tray("combat").is_some());
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_creates_custom_die_from_command_faces() {
    let path = unique_path("die");
    let mut app = App::load_from_path(path.clone()).unwrap();

    app.apply_command(Command::DiceNew {
        name: "fate".into(),
        faces: vec!["-1".into(), "0".into(), "+".into()],
    })
    .unwrap();

    let die = app.engine.resolve_die_name("fate").unwrap();
    assert_eq!(die, format!("{CUSTOM_PREFIX}fate"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_toggles_overview_selection_by_page_id() {
    let path = unique_path("selection");
    let mut app = App::load_from_path(path.clone()).unwrap();
    app.engine.create_tray("combat").unwrap();
    app.engine.create_tray("loot").unwrap();

    app.toggle_tray_selection(1).unwrap();
    assert_eq!(app.selected_trays, vec!["combat"]);

    app.toggle_tray_selection(1).unwrap();
    assert!(app.selected_trays.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_opens_tray_by_page_id_and_toggles_slot_lock() {
    let path = unique_path("slot");
    let mut app = App::load_from_path(path.clone()).unwrap();
    app.engine.create_tray("combat").unwrap();
    app.engine.add_die_to_tray("d6", "combat").unwrap();

    app.open_tray_by_page_id(1).unwrap();
    assert_eq!(app.screen, Screen::TrayDetail("combat".into()));

    app.toggle_slot_lock(1).unwrap();
    assert!(app.engine.get_tray("combat").unwrap().slots[0].locked);
    let _ = std::fs::remove_file(path);
}

#[test]
fn face_parser_keeps_numbers_numeric_and_others_text() {
    assert_eq!(App::parse_face("-2"), FaceValue::Integer(-2));
    assert_eq!(App::parse_face("heads"), FaceValue::Text("heads".into()));
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p rdice-tui --test app_test
```

Expected: compile failure for missing app methods.

- [ ] **Step 3: Implement app actions**

Add these methods to `impl App` in `crates/rdice-tui/src/app.rs`:

```rust
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};

use crate::command::Command;

impl App {
    pub fn apply_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::ManagerDice => self.screen = Screen::DiceManager,
            Command::ManagerTrays => self.screen = Screen::TrayManager,
            Command::Overview => self.screen = Screen::Overview,
            Command::OpenTray(name) => self.open_tray(name)?,
            Command::DiceNew { name, faces } => {
                let faces = faces.iter().map(|face| Self::parse_face(face)).collect();
                self.engine.create_die(&name, faces)?;
                self.save()?;
            }
            Command::DiceDelete(name) => {
                self.engine.delete_die(&name)?;
                self.save()?;
            }
            Command::TrayNew(name) => {
                self.engine.create_tray(&name)?;
                self.save()?;
            }
            Command::TrayDelete(name) => {
                self.engine.delete_tray(&name)?;
                self.selected_trays.retain(|selected| selected != &name);
                self.save()?;
            }
            Command::Quit => self.should_quit = true,
        }
        Ok(())
    }

    pub fn parse_face(face: &str) -> FaceValue {
        face.parse::<i64>()
            .map(FaceValue::Integer)
            .unwrap_or_else(|_| FaceValue::Text(face.to_string()))
    }

    pub fn toggle_tray_selection(&mut self, page_id: usize) -> Result<()> {
        let name = self.tray_name_for_page_id(page_id)?;
        if self.selected_trays.iter().any(|selected| selected == &name) {
            self.selected_trays.retain(|selected| selected != &name);
        } else {
            self.selected_trays.push(name);
        }
        Ok(())
    }

    pub fn open_tray_by_page_id(&mut self, page_id: usize) -> Result<()> {
        let name = self.tray_name_for_page_id(page_id)?;
        self.open_tray(name)
    }

    pub fn open_tray(&mut self, name: String) -> Result<()> {
        if self.engine.get_tray(&name).is_none() {
            return Err(DiceError::TrayNotFound(name));
        }
        self.screen = Screen::TrayDetail(name);
        Ok(())
    }

    pub fn roll_selected_trays(&mut self) -> Result<()> {
        if self.selected_trays.is_empty() {
            self.message = Some("select one or more trays first".to_string());
            return Ok(());
        }

        for tray_name in self.selected_trays.clone() {
            self.engine.roll_tray(&tray_name)?;
        }
        self.save()
    }

    pub fn roll_current_tray(&mut self) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        self.engine.roll_tray(&tray_name)?;
        self.save()
    }

    pub fn toggle_slot_lock(&mut self, slot_id: u32) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        let locked = self
            .engine
            .get_tray(&tray_name)
            .and_then(|tray| tray.slots.iter().find(|slot| slot.slot_id == slot_id))
            .map(|slot| slot.locked)
            .ok_or_else(|| DiceError::SlotNotFound {
                tray: tray_name.clone(),
                slot_id,
            })?;

        if locked {
            self.engine.unlock_slot(&tray_name, slot_id)?;
        } else {
            self.engine.lock_slot(&tray_name, slot_id)?;
        }
        self.save()
    }

    pub fn remove_slot(&mut self, slot_id: u32) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        self.engine.remove_slot(&tray_name, slot_id)?;
        self.save()
    }

    pub fn add_die_to_current_tray(&mut self, die_name: &str) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        self.engine.add_die_to_tray(die_name, &tray_name)?;
        self.save()
    }

    fn tray_name_for_page_id(&self, page_id: usize) -> Result<String> {
        if !(1..=9).contains(&page_id) {
            return Err(DiceError::StorageError(format!("invalid page id: {page_id}")));
        }
        let index = self.overview_page * 9 + page_id - 1;
        self.engine
            .list_trays()
            .get(index)
            .map(|tray| tray.name.clone())
            .ok_or_else(|| DiceError::StorageError(format!("no tray for page id: {page_id}")))
    }

    fn current_tray_name(&self) -> Result<String> {
        match &self.screen {
            Screen::TrayDetail(name) | Screen::AddDie(name) => Ok(name.clone()),
            _ => Err(DiceError::StorageError(
                "current screen is not a tray".to_string(),
            )),
        }
    }
}
```

- [ ] **Step 4: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui --test app_test
```

Expected: app tests pass.

---

### Task 4: Overview View Model And Formatting

**Files:**
- Modify `crates/rdice-tui/src/screens/overview.rs`
- Create `crates/rdice-tui/tests/overview_test.rs`

- [ ] **Step 1: Write overview tests**

Create `crates/rdice-tui/tests/overview_test.rs`:

```rust
use rdice_core::DiceEngine;
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_tui::screens::overview::{
    OverviewOptions, build_tray_cards, format_ev, format_total_line,
};

#[test]
fn ev_format_rounds_to_one_decimal() {
    assert_eq!(format_ev(12.04), "12.0");
    assert_eq!(format_ev(12.05), "12.1");
    assert_eq!(format_ev(3.5), "3.5");
}

#[test]
fn total_line_formats_optional_analysis_and_text_marker() {
    assert_eq!(
        format_total_line(Some(8), Some((2, 24)), Some(12.04), true),
        "Total(2-24~12.0):8 +t"
    );
    assert_eq!(
        format_total_line(Some(8), Some((2, 24)), None, false),
        "Total(2-24):8"
    );
    assert_eq!(
        format_total_line(None, None, Some(12.05), false),
        "Total(~12.1):-"
    );
}

#[test]
fn tray_card_groups_numeric_dice_and_marks_hidden_text() {
    let mut engine = DiceEngine::new();
    engine
        .create_die("weather_long_name", vec![FaceValue::Text("rain".into())])
        .unwrap();
    engine.create_tray("travel").unwrap();
    engine.add_die_to_tray("d6", "travel").unwrap();
    engine.add_die_to_tray("d6", "travel").unwrap();
    engine.add_die_to_tray("weather_long_name", "travel").unwrap();
    engine.roll_tray("travel").unwrap();

    let cards = build_tray_cards(
        &engine,
        &[],
        0,
        OverviewOptions {
            text_visible: false,
            range_visible: true,
            ev_visible: true,
        },
    )
    .unwrap();

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].page_id, 1);
    assert!(cards[0].composition.contains("2xd6"));
    assert!(cards[0].composition.contains("weather..."));
    assert!(cards[0].total_line.contains("+t"));
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p rdice-tui --test overview_test
```

Expected: compile failure for missing overview view-model functions.

- [ ] **Step 3: Implement overview view model**

Implement in `crates/rdice-tui/src/screens/overview.rs`:

```rust
use std::collections::BTreeMap;

use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_core::engine::DiceEngine;
use rdice_core::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverviewOptions {
    pub text_visible: bool,
    pub range_visible: bool,
    pub ev_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayCard {
    pub page_id: usize,
    pub name: String,
    pub selected: bool,
    pub composition: String,
    pub total_line: String,
    pub text_line: Option<String>,
}

pub fn build_tray_cards(
    engine: &DiceEngine,
    selected_trays: &[String],
    page: usize,
    options: OverviewOptions,
) -> Result<Vec<TrayCard>> {
    let start = page * 9;
    let trays = engine.list_trays().iter().skip(start).take(9);
    let mut cards = Vec::new();

    for (offset, tray) in trays.enumerate() {
        let die_names = tray
            .slots
            .iter()
            .map(|slot| slot.die_name.clone())
            .collect::<Vec<_>>();
        let analysis = engine.analyze_roll(&die_names, &[])?;
        let numeric_total = tray
            .slots
            .iter()
            .filter_map(|slot| match &slot.current_value {
                Some(FaceValue::Integer(value)) => Some(*value),
                _ => None,
            })
            .reduce(|acc, value| acc + value);
        let text_values = tray
            .slots
            .iter()
            .filter_map(|slot| match &slot.current_value {
                Some(FaceValue::Text(value)) => Some(value.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let range = options
            .range_visible
            .then_some((analysis.point_range.min, analysis.point_range.max));
        let ev = options.ev_visible.then_some(analysis.expected_value);

        cards.push(TrayCard {
            page_id: offset + 1,
            name: tray.name.clone(),
            selected: selected_trays.iter().any(|selected| selected == &tray.name),
            composition: format_composition(&die_names),
            total_line: format_total_line(
                numeric_total,
                range,
                ev,
                !options.text_visible && !text_values.is_empty(),
            ),
            text_line: (options.text_visible && !text_values.is_empty())
                .then(|| format!("Text:{}", text_values.join(","))),
        });
    }

    Ok(cards)
}

pub fn format_total_line(
    total: Option<i64>,
    range: Option<(i64, i64)>,
    ev: Option<f64>,
    hidden_text: bool,
) -> String {
    let analysis = match (range, ev) {
        (Some((min, max)), Some(ev)) => format!("({min}-{max}~{})", format_ev(ev)),
        (Some((min, max)), None) => format!("({min}-{max})"),
        (None, Some(ev)) => format!("(~{})", format_ev(ev)),
        (None, None) => String::new(),
    };
    let total = total
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let suffix = if hidden_text { " +t" } else { "" };
    format!("Total{analysis}:{total}{suffix}")
}

pub fn format_ev(value: f64) -> String {
    format!("{value:.1}")
}

fn format_composition(die_names: &[String]) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for die_name in die_names {
        *counts.entry(display_die_name(die_name)).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(name, count)| {
            if count == 1 {
                name
            } else {
                format!("{count}x{name}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_die_name(name: &str) -> String {
    let stripped = name.strip_prefix(CUSTOM_PREFIX).unwrap_or(name);
    let stripped = if stripped.len() > 1
        && stripped.starts_with('D')
        && stripped[1..].chars().all(|ch| ch.is_ascii_digit())
    {
        stripped.to_ascii_lowercase()
    } else {
        stripped.to_string()
    };
    let mut chars = stripped.chars();
    let prefix = chars.by_ref().take(7).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}...")
    } else {
        prefix
    }
}
```

- [ ] **Step 4: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui --test overview_test
```

Expected: overview tests pass.

---

### Task 5: Input Sequence Handling

**Files:**
- Modify `crates/rdice-tui/src/input.rs`
- Create `crates/rdice-tui/tests/input_test.rs`

- [ ] **Step 1: Write input tests**

Create `crates/rdice-tui/tests/input_test.rs`:

```rust
use rdice_tui::input::{InputAction, InputState};

#[test]
fn overview_digit_selects_tray() {
    let mut state = InputState::default();
    assert_eq!(state.push('3'), Some(InputAction::ToggleTray(3)));
}

#[test]
fn overview_o_digit_opens_tray() {
    let mut state = InputState::default();
    assert_eq!(state.push('o'), None);
    assert_eq!(state.push('3'), Some(InputAction::OpenTray(3)));
}

#[test]
fn tray_l_digit_locks_slot() {
    let mut state = InputState::default();
    assert_eq!(state.push('l'), None);
    assert_eq!(state.push('3'), Some(InputAction::ToggleSlotLock(3)));
}

#[test]
fn tray_d_digit_removes_slot() {
    let mut state = InputState::default();
    assert_eq!(state.push('d'), None);
    assert_eq!(state.push('3'), Some(InputAction::RemoveSlot(3)));
}

#[test]
fn command_key_enters_command_mode() {
    let mut state = InputState::default();
    assert_eq!(state.push(':'), Some(InputAction::EnterCommandMode));
}
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p rdice-tui --test input_test
```

Expected: compile failure for missing input types.

- [ ] **Step 3: Implement input state**

Replace `crates/rdice-tui/src/input.rs` with:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    ToggleTray(usize),
    OpenTray(usize),
    Roll,
    ToggleText,
    ToggleRange,
    ToggleEv,
    PreviousPage,
    NextPage,
    EnterCommandMode,
    Quit,
    Escape,
    AddDie,
    ToggleSlotLock(u32),
    RemoveSlot(u32),
}

#[derive(Debug, Default)]
pub struct InputState {
    pending_prefix: Option<char>,
}

impl InputState {
    pub fn push(&mut self, ch: char) -> Option<InputAction> {
        if let Some(prefix) = self.pending_prefix.take() {
            return match (prefix, ch.to_digit(10)) {
                ('o', Some(value @ 1..=9)) => Some(InputAction::OpenTray(value as usize)),
                ('l', Some(value @ 1..=9)) => Some(InputAction::ToggleSlotLock(value)),
                ('d', Some(value @ 1..=9)) => Some(InputAction::RemoveSlot(value)),
                _ => None,
            };
        }

        match ch {
            '1'..='9' => Some(InputAction::ToggleTray(ch.to_digit(10).unwrap() as usize)),
            'o' | 'l' | 'd' => {
                self.pending_prefix = Some(ch);
                None
            }
            'r' => Some(InputAction::Roll),
            't' => Some(InputAction::ToggleText),
            'R' => Some(InputAction::ToggleRange),
            'E' => Some(InputAction::ToggleEv),
            ':' => Some(InputAction::EnterCommandMode),
            'q' => Some(InputAction::Quit),
            'a' => Some(InputAction::AddDie),
            _ => None,
        }
    }

    pub fn clear(&mut self) {
        self.pending_prefix = None;
    }
}
```

- [ ] **Step 4: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui --test input_test
```

Expected: input tests pass.

---

### Task 6: Ratatui Rendering

**Files:**
- Modify `crates/rdice-tui/src/ui.rs`
- Modify `crates/rdice-tui/src/screens/overview.rs`
- Modify `crates/rdice-tui/src/screens/tray.rs`
- Modify `crates/rdice-tui/src/screens/dice_manager.rs`
- Modify `crates/rdice-tui/src/screens/tray_manager.rs`

- [ ] **Step 1: Add root render function**

In `crates/rdice-tui/src/ui.rs`, implement:

```rust
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::{App, Screen};
use crate::screens::{dice_manager, overview, tray, tray_manager};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    match &app.screen {
        Screen::Overview => overview::render(frame, chunks[0], app),
        Screen::TrayDetail(name) => tray::render(frame, chunks[0], app, name),
        Screen::AddDie(name) => tray::render_add_die(frame, chunks[0], app, name),
        Screen::DiceManager => dice_manager::render(frame, chunks[0], app),
        Screen::TrayManager => tray_manager::render(frame, chunks[0], app),
    }

    let status = app
        .command_buffer
        .as_ref()
        .map(|buffer| format!(":{buffer}"))
        .or_else(|| app.message.clone())
        .unwrap_or_else(|| {
            "1-9 select  o<num> open  r roll  t text  R range  E ev  PgUp/PgDn  :  q".into()
        });
    frame.render_widget(Paragraph::new(status), chunks[1]);
}
```

- [ ] **Step 2: Add overview renderer**

Append this to `crates/rdice-tui/src/screens/overview.rs`:

```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let options = OverviewOptions {
        text_visible: app.overview_text_visible,
        range_visible: app.overview_range_visible,
        ev_visible: app.overview_ev_visible,
    };
    let cards = build_tray_cards(
        &app.engine,
        &app.selected_trays,
        app.overview_page,
        options,
    )
    .unwrap_or_default();

    frame.render_widget(Paragraph::new(render_grid_text(&cards, area.width)), area);
}

pub fn render_grid_text(cards: &[TrayCard], width: u16) -> String {
    let columns = 3_usize;
    let cell_width = ((width as usize).saturating_sub(columns + 1) / columns).max(18);
    let border = format!(
        "+{}+",
        std::iter::repeat("-".repeat(cell_width))
            .take(columns)
            .collect::<Vec<_>>()
            .join("+")
    );
    let mut output = vec![border.clone()];

    for row in 0..3 {
        let row_cards = (0..columns)
            .map(|column| cards.get(row * columns + column))
            .collect::<Vec<_>>();
        for line_index in 0..4 {
            let cells = row_cards
                .iter()
                .map(|card| {
                    let text = card
                        .map(|card| card_line(card, line_index, cell_width))
                        .unwrap_or_default();
                    pad_cell(&text, cell_width)
                })
                .collect::<Vec<_>>()
                .join("|");
            output.push(format!("|{cells}|"));
        }
        output.push(border.clone());
    }

    output.join("\n")
}

fn card_line(card: &TrayCard, line_index: usize, width: usize) -> String {
    match line_index {
        0 => {
            let marker = if card.selected { " *" } else { "" };
            format!("[{}] {}{}", card.page_id, card.name, marker)
        }
        1 => card.composition.clone(),
        2 => right_align_text_marker(&card.total_line, width),
        3 => card.text_line.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

fn right_align_text_marker(line: &str, width: usize) -> String {
    let Some(base) = line.strip_suffix(" +t") else {
        return line.to_string();
    };
    if base.len() + 3 >= width {
        return line.to_string();
    }
    format!("{base}{}+t", " ".repeat(width - base.len() - 2))
}

fn pad_cell(text: &str, width: usize) -> String {
    let truncated = truncate(text, width);
    format!("{truncated:<width$}")
}

fn truncate(text: &str, width: usize) -> String {
    let mut output = String::new();
    for ch in text.chars() {
        if output.len() + ch.len_utf8() > width {
            break;
        }
        output.push(ch);
    }
    output
}
```

- [ ] **Step 3: Add tray renderers**

Implement `crates/rdice-tui/src/screens/tray.rs`:

```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App, tray_name: &str) {
    let body = app
        .engine
        .get_tray(tray_name)
        .map(|tray| {
            tray.slots
                .iter()
                .map(|slot| {
                    let lock = if slot.locked { " LOCK" } else { "" };
                    let value = slot
                        .current_value
                        .as_ref()
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    format!("[{}] {}{} value {}", slot.slot_id, slot.die_name, lock, value)
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|| "Tray not found".to_string());

    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(format!(" Tray: {tray_name} "))
                .borders(Borders::ALL),
        ),
        area,
    );
}

pub fn render_add_die(frame: &mut Frame<'_>, area: Rect, app: &App, tray_name: &str) {
    let body = app
        .engine
        .list_dice()
        .iter()
        .take(9)
        .enumerate()
        .map(|(index, die)| format!("[{}] {}", index + 1, die.name))
        .collect::<Vec<_>>()
        .join("\n");

    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(format!(" Add die to {tray_name} "))
                .borders(Borders::ALL),
        ),
        area,
    );
}
```

- [ ] **Step 4: Add manager renderers**

Implement `crates/rdice-tui/src/screens/dice_manager.rs`:

```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let body = app
        .engine
        .custom_dice()
        .into_iter()
        .map(|die| format!("{}: {} faces", die.name, die.faces.len()))
        .collect::<Vec<_>>()
        .join("\n");
    frame.render_widget(
        Paragraph::new(body).block(Block::default().title(" Dice Manager ").borders(Borders::ALL)),
        area,
    );
}
```

Implement `crates/rdice-tui/src/screens/tray_manager.rs`:

```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let body = app
        .engine
        .list_trays()
        .iter()
        .map(|tray| format!("{}: {} slots", tray.name, tray.slots.len()))
        .collect::<Vec<_>>()
        .join("\n");
    frame.render_widget(
        Paragraph::new(body).block(Block::default().title(" Tray Manager ").borders(Borders::ALL)),
        area,
    );
}
```

- [ ] **Step 5: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui
```

Expected: all `rdice-tui` tests pass.

---

### Task 7: Terminal Event Loop

**Files:**
- Modify `crates/rdice-tui/src/main.rs`
- Modify `crates/rdice-tui/src/app.rs`

- [ ] **Step 1: Add app helpers for toggles and paging**

Add to `impl App`:

```rust
pub fn previous_page(&mut self) {
    self.overview_page = self.overview_page.saturating_sub(1);
}

pub fn next_page(&mut self) {
    let tray_count = self.engine.list_trays().len();
    let max_page = tray_count.saturating_sub(1) / 9;
    self.overview_page = (self.overview_page + 1).min(max_page);
}

pub fn toggle_text_visible(&mut self) {
    self.overview_text_visible = !self.overview_text_visible;
}

pub fn toggle_range_visible(&mut self) {
    self.overview_range_visible = !self.overview_range_visible;
}

pub fn toggle_ev_visible(&mut self) {
    self.overview_ev_visible = !self.overview_ev_visible;
}

pub fn enter_command_mode(&mut self) {
    self.command_buffer = Some(String::new());
}

pub fn leave_command_mode(&mut self) {
    self.command_buffer = None;
}
```

- [ ] **Step 2: Replace `main.rs` with raw terminal loop**

Implement terminal setup and cleanup in `crates/rdice-tui/src/main.rs`:

```rust
use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rdice_core::error::{DiceError, Result};
use rdice_tui::app::App;
use rdice_tui::command::parse_command;
use rdice_tui::input::{InputAction, InputState};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::load_default()?;
    let mut input = InputState::default();

    while !app.should_quit {
        terminal
            .draw(|frame| rdice_tui::ui::render(frame, &app))
            .map_err(to_storage_error)?;

        if event::poll(Duration::from_millis(100)).map_err(to_storage_error)? {
            if let Event::Key(key) = event::read().map_err(to_storage_error)? {
                handle_key(&mut app, &mut input, key)?;
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode().map_err(to_storage_error)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(to_storage_error)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(to_storage_error)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode().map_err(to_storage_error)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(to_storage_error)?;
    terminal.show_cursor().map_err(to_storage_error)?;
    Ok(())
}

fn to_storage_error(err: impl std::fmt::Display) -> DiceError {
    DiceError::StorageError(err.to_string())
}

fn handle_key(app: &mut App, input: &mut InputState, key: KeyEvent) -> Result<()> {
    if let Some(buffer) = &mut app.command_buffer {
        match key.code {
            KeyCode::Esc => app.leave_command_mode(),
            KeyCode::Enter => {
                let command_input = buffer.clone();
                app.leave_command_mode();
                match parse_command(&command_input).and_then(|command| app.apply_command(command)) {
                    Ok(()) => app.message = None,
                    Err(err) => app.message = Some(err.to_string()),
                }
            }
            KeyCode::Backspace => {
                buffer.pop();
            }
            KeyCode::Char(ch) => buffer.push(ch),
            _ => {}
        }
        return Ok(());
    }

    let action = match key.code {
        KeyCode::Esc => Some(InputAction::Escape),
        KeyCode::PageUp => Some(InputAction::PreviousPage),
        KeyCode::PageDown => Some(InputAction::NextPage),
        KeyCode::Char(ch) => input.push(ch),
        _ => None,
    };

    if let Some(action) = action {
        apply_input_action(app, action)?;
    }

    Ok(())
}

fn apply_input_action(app: &mut App, action: InputAction) -> Result<()> {
    match action {
        InputAction::ToggleTray(id) => app.toggle_tray_selection(id)?,
        InputAction::OpenTray(id) => app.open_tray_by_page_id(id)?,
        InputAction::Roll => app.roll_selected_trays()?,
        InputAction::ToggleText => app.toggle_text_visible(),
        InputAction::ToggleRange => app.toggle_range_visible(),
        InputAction::ToggleEv => app.toggle_ev_visible(),
        InputAction::PreviousPage => app.previous_page(),
        InputAction::NextPage => app.next_page(),
        InputAction::EnterCommandMode => app.enter_command_mode(),
        InputAction::Quit => app.should_quit = true,
        InputAction::Escape => app.screen = rdice_tui::app::Screen::Overview,
        InputAction::AddDie => {
            if let rdice_tui::app::Screen::TrayDetail(name) = &app.screen {
                app.screen = rdice_tui::app::Screen::AddDie(name.clone());
            }
        }
        InputAction::ToggleSlotLock(slot_id) => app.toggle_slot_lock(slot_id)?,
        InputAction::RemoveSlot(slot_id) => app.remove_slot(slot_id)?,
    }
    Ok(())
}
```

- [ ] **Step 3: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui
cargo run -p rdice-tui
```

Expected:

- Tests pass.
- `cargo run -p rdice-tui` opens the alternate-screen TUI.
- `q` exits and restores the terminal.

---

### Task 8: Add Die Flow And Screen-Specific Input Semantics

**Files:**
- Modify `crates/rdice-tui/src/app.rs`
- Modify `crates/rdice-tui/src/main.rs`
- Modify `crates/rdice-tui/src/screens/tray.rs`
- Modify `crates/rdice-tui/tests/app_test.rs`

- [ ] **Step 1: Add add-die page ID helper test**

Append to `crates/rdice-tui/tests/app_test.rs`:

```rust
#[test]
fn add_die_by_page_id_adds_to_current_tray() {
    let path = unique_path("add-die");
    let mut app = App::load_from_path(path.clone()).unwrap();
    app.engine.create_tray("combat").unwrap();
    app.screen = Screen::AddDie("combat".into());

    app.add_die_by_page_id(1).unwrap();

    let tray = app.engine.get_tray("combat").unwrap();
    assert_eq!(tray.slots.len(), 1);
    assert_eq!(tray.slots[0].die_name, "d4");
    let _ = std::fs::remove_file(path);
}
```

- [ ] **Step 2: Implement add-die helper**

Add to `impl App`:

```rust
pub fn add_die_by_page_id(&mut self, page_id: usize) -> Result<()> {
    if !(1..=9).contains(&page_id) {
        return Err(DiceError::StorageError(format!("invalid page id: {page_id}")));
    }
    let die_name = self
        .engine
        .list_dice()
        .get(page_id - 1)
        .map(|die| die.name.clone())
        .ok_or_else(|| DiceError::StorageError(format!("no die for page id: {page_id}")))?;
    self.add_die_to_current_tray(&die_name)
}
```

- [ ] **Step 3: Make numeric input screen-specific**

In `apply_input_action` in `main.rs`, handle `InputAction::ToggleTray(id)` based on screen:

```rust
InputAction::ToggleTray(id) => match app.screen {
    rdice_tui::app::Screen::Overview => app.toggle_tray_selection(id)?,
    rdice_tui::app::Screen::AddDie(_) => app.add_die_by_page_id(id)?,
    _ => {}
},
```

Keep other action arms unchanged.

- [ ] **Step 4: Verify**

Run:

```sh
cargo fmt
cargo test -p rdice-tui
```

Expected: all tests pass.

---

### Task 9: Final Verification And Documentation

**Files:**
- Modify `README.md`
- Modify `README_zh.md`

- [ ] **Step 1: Document TUI usage**

Add concise TUI sections to both README files. Include:

```text
rdice-tui
```

And the key bindings:

```text
Overview: 1-9 select, o<num> open, r roll, t/R/E toggle display, PgUp/PgDn page, : command, q quit
Tray Detail: r roll, l<num> lock/unlock, d<num> remove, a add die, Esc overview
Command mode: :manager dice, :manager trays, :dice new, :dice delete, :tray new, :tray delete
```

- [ ] **Step 2: Run full verification**

Run:

```sh
cargo fmt
cargo test
cargo run -p rdice-tui
```

Expected:

- Full workspace tests pass.
- TUI opens and quits cleanly with `q`.
- Terminal returns to normal after quit.

- [ ] **Step 3: Manual smoke checklist**

Run `cargo run -p rdice-tui` and verify:

```text
:tray new combat
:dice new fate -1 0 +
o1
a
1
r
l1
d1
Esc
q
```

Expected:

- Tray can be created.
- Custom die can be created.
- Tray can be opened.
- Die can be added.
- Tray can be rolled.
- Slot can be locked/unlocked.
- Slot can be removed.
- Overview returns with `Esc`.
- App quits with `q`.
