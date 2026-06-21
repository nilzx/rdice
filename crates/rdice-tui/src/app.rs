use std::path::PathBuf;

use rdice_core::DiceEngine;
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_core::error::{DiceError, Result};

use crate::command::Command;
use crate::storage;

const DEFAULT_TRAY_NAME: &str = "default";

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
    pub manager_return: Option<Screen>,
    pub should_quit: bool,
}

impl App {
    pub fn load_default() -> Result<Self> {
        let state_path = default_state_path()?;
        Self::load_from_path(state_path)
    }

    pub fn load_from_path(state_path: PathBuf) -> Result<Self> {
        let mut engine = match storage::load(&state_path) {
            Ok(engine) => engine,
            Err(err) => {
                eprintln!("Warning: failed to load TUI state: {err}");
                DiceEngine::new()
            }
        };

        let needs_default_tray = engine.list_trays().is_empty();
        if needs_default_tray {
            engine.create_tray(DEFAULT_TRAY_NAME)?;
        }

        let app = Self::new(engine, state_path);
        if needs_default_tray {
            app.save()?;
        }

        Ok(app)
    }

    pub fn new(engine: DiceEngine, state_path: PathBuf) -> Self {
        Self {
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
            manager_return: None,
            should_quit: false,
        }
    }

    pub fn save(&self) -> Result<()> {
        storage::save(&self.state_path, &self.engine)
    }

    pub fn apply_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::ManagerDice => self.open_dice_manager(),
            Command::ManagerTrays => self.open_tray_manager(),
            Command::Overview => {
                self.manager_return = None;
                self.screen = Screen::Overview;
            }
            Command::OpenTray(name) => self.open_tray(name)?,
            Command::DiceNew { name, faces } => {
                let faces = faces.iter().map(|face| Self::parse_face(face)).collect();
                self.engine.create_die(&name, faces)?;
                self.save()?;
                self.message = Some(format!("created die {name}"));
            }
            Command::DiceDelete(name) => {
                self.engine.delete_die(&name)?;
                self.save()?;
                self.message = Some(format!("deleted die {name}"));
            }
            Command::DiceEdit { name, faces } => {
                let faces = faces.iter().map(|face| Self::parse_face(face)).collect();
                self.engine.modify_die(&name, faces)?;
                self.save()?;
                self.message = Some(format!("edited die {name}"));
            }
            Command::TrayNew(name) => {
                self.engine.create_tray(&name)?;
                self.save()?;
                self.message = Some(format!("created tray {name}"));
            }
            Command::TrayDelete(name) => {
                self.engine.delete_tray(&name)?;
                self.selected_trays.retain(|selected| selected != &name);
                self.save()?;
                self.message = Some(format!("deleted tray {name}"));
            }
            Command::TrayRename { old_name, new_name } => {
                self.engine.rename_tray(&old_name, &new_name)?;
                for selected in &mut self.selected_trays {
                    if selected == &old_name {
                        *selected = new_name.clone();
                    }
                }
                if matches!(&self.screen, Screen::TrayDetail(name) | Screen::AddDie(name) if name == &old_name)
                {
                    self.screen = Screen::TrayDetail(new_name.clone());
                }
                self.save()?;
                self.message = Some(format!("renamed tray {old_name} to {new_name}"));
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
            self.message = Some(format!("deselected tray {name}"));
        } else {
            self.message = Some(format!("selected tray {name}"));
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
        self.manager_return = None;
        self.message = None;
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

        self.save()?;
        self.message = Some(format!("rolled {} tray(s)", self.selected_trays.len()));
        Ok(())
    }

    pub fn roll_current_tray(&mut self) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        self.engine.roll_tray(&tray_name)?;
        self.save()?;
        self.message = Some(format!("rolled tray {tray_name}"));
        Ok(())
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

        self.save()?;
        let action = if locked { "unlocked" } else { "locked" };
        self.message = Some(format!("{action} slot {slot_id}"));
        Ok(())
    }

    pub fn remove_slot(&mut self, slot_id: u32) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        self.engine.remove_slot(&tray_name, slot_id)?;
        self.save()?;
        self.message = Some(format!("removed slot {slot_id}"));
        Ok(())
    }

    pub fn add_die_to_current_tray(&mut self, die_name: &str) -> Result<()> {
        let tray_name = self.current_tray_name()?;
        self.engine.add_die_to_tray(die_name, &tray_name)?;
        self.save()?;
        self.message = Some(format!("added {die_name} to {tray_name}"));
        Ok(())
    }

    pub fn add_die_by_page_id(&mut self, page_id: usize) -> Result<()> {
        if !(1..=9).contains(&page_id) {
            return Err(DiceError::InvalidArguments(format!(
                "invalid page id: {page_id}"
            )));
        }

        let die_name = self
            .engine
            .list_dice()
            .get(page_id - 1)
            .map(|die| die.name.clone())
            .ok_or_else(|| DiceError::InvalidArguments(format!("no die for page id: {page_id}")))?;
        self.add_die_to_current_tray(&die_name)
    }

    pub fn previous_page(&mut self) {
        self.overview_page = self.overview_page.saturating_sub(1);
        self.message = Some(format!("page {}", self.overview_page + 1));
    }

    pub fn next_page(&mut self) {
        let tray_count = self.engine.list_trays().len();
        let max_page = tray_count.saturating_sub(1) / 9;
        self.overview_page = (self.overview_page + 1).min(max_page);
        self.message = Some(format!("page {}", self.overview_page + 1));
    }

    pub fn toggle_text_visible(&mut self) {
        self.overview_text_visible = !self.overview_text_visible;
        let state = if self.overview_text_visible {
            "on"
        } else {
            "off"
        };
        self.message = Some(format!("text {state}"));
    }

    pub fn toggle_range_visible(&mut self) {
        self.overview_range_visible = !self.overview_range_visible;
        let state = if self.overview_range_visible {
            "on"
        } else {
            "off"
        };
        self.message = Some(format!("range {state}"));
    }

    pub fn toggle_ev_visible(&mut self) {
        self.overview_ev_visible = !self.overview_ev_visible;
        let state = if self.overview_ev_visible {
            "on"
        } else {
            "off"
        };
        self.message = Some(format!("ev {state}"));
    }

    pub fn enter_command_mode(&mut self) {
        self.command_buffer = Some(String::new());
    }

    pub fn prefill_command(&mut self, command: impl Into<String>) {
        self.command_buffer = Some(command.into());
    }

    pub fn leave_command_mode(&mut self) {
        self.command_buffer = None;
    }

    pub fn open_context_manager(&mut self) {
        match &self.screen {
            Screen::Overview => self.open_tray_manager(),
            Screen::TrayDetail(_) => self.open_dice_manager(),
            _ => {}
        }
    }

    pub fn open_tray_manager(&mut self) {
        self.manager_return = Some(self.screen.clone());
        self.screen = Screen::TrayManager;
        self.message = Some("tray manager".to_string());
    }

    pub fn open_dice_manager(&mut self) {
        self.manager_return = Some(self.screen.clone());
        self.screen = Screen::DiceManager;
        self.message = Some("dice manager".to_string());
    }

    pub fn escape(&mut self) {
        self.screen = match &self.screen {
            Screen::AddDie(tray_name) => Screen::TrayDetail(tray_name.clone()),
            Screen::TrayDetail(_) => Screen::Overview,
            Screen::DiceManager | Screen::TrayManager => {
                self.manager_return.take().unwrap_or(Screen::Overview)
            }
            Screen::Overview => Screen::Overview,
        };
        self.message = None;
    }

    pub fn prefill_new_target(&mut self) -> Result<()> {
        match &self.screen {
            Screen::TrayManager => self.prefill_command("tray new "),
            Screen::DiceManager => self.prefill_command("dice new "),
            _ => {
                return Err(DiceError::InvalidArguments(
                    "new is only available in manager pages".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn prefill_delete_target(&mut self, page_id: usize) -> Result<()> {
        match &self.screen {
            Screen::TrayManager => {
                let tray_name = self.tray_name_for_page_id(page_id)?;
                self.prefill_command(format!("tray delete {tray_name}"));
            }
            Screen::DiceManager => {
                let die_name = self.custom_die_name_for_page_id(page_id)?;
                self.prefill_command(format!("dice delete {die_name}"));
            }
            _ => {
                return Err(DiceError::InvalidArguments(
                    "delete is only available in manager pages".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn prefill_edit_target(&mut self, page_id: usize) -> Result<()> {
        match &self.screen {
            Screen::TrayManager => {
                let tray_name = self.tray_name_for_page_id(page_id)?;
                self.prefill_command(format!("tray rename {tray_name} "));
            }
            Screen::DiceManager => {
                let die_name = self.custom_die_name_for_page_id(page_id)?;
                self.prefill_command(format!("dice edit {die_name} "));
            }
            _ => {
                return Err(DiceError::InvalidArguments(
                    "edit is only available in manager pages".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn tray_name_for_page_id(&self, page_id: usize) -> Result<String> {
        if !(1..=9).contains(&page_id) {
            return Err(DiceError::InvalidArguments(format!(
                "invalid page id: {page_id}"
            )));
        }

        let index = self.overview_page * 9 + page_id - 1;
        self.engine
            .list_trays()
            .get(index)
            .map(|tray| tray.name.clone())
            .ok_or_else(|| DiceError::InvalidArguments(format!("no tray for page id: {page_id}")))
    }

    fn current_tray_name(&self) -> Result<String> {
        match &self.screen {
            Screen::TrayDetail(name) | Screen::AddDie(name) => Ok(name.clone()),
            _ => Err(DiceError::InvalidArguments(
                "current screen is not a tray".to_string(),
            )),
        }
    }

    fn custom_die_name_for_page_id(&self, page_id: usize) -> Result<String> {
        if !(1..=9).contains(&page_id) {
            return Err(DiceError::InvalidArguments(format!(
                "invalid page id: {page_id}"
            )));
        }

        self.engine
            .custom_dice()
            .get(page_id - 1)
            .map(|die| display_custom_name(&die.name))
            .ok_or_else(|| DiceError::InvalidArguments(format!("no die for page id: {page_id}")))
    }
}

fn display_custom_name(name: &str) -> String {
    name.strip_prefix(CUSTOM_PREFIX).unwrap_or(name).to_string()
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
