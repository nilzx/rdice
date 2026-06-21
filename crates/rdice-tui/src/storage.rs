use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use rdice_core::die::{Die, DieKind, FaceValue};
use rdice_core::engine::{DiceEngine, TrayResult};
use rdice_core::error::{DiceError, Result};
use rdice_core::tray::Tray;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollHistoryEntry {
    pub tray_name: String,
    pub total: Option<i64>,
    pub slots: Vec<RollHistorySlot>,
}

impl From<TrayResult> for RollHistoryEntry {
    fn from(result: TrayResult) -> Self {
        Self {
            tray_name: result.tray_name,
            total: result.integer_sum,
            slots: result
                .slots
                .into_iter()
                .map(|slot| RollHistorySlot {
                    slot_id: slot.slot_id,
                    die_name: slot.die_name,
                    locked: slot.locked,
                    value: slot.current_value,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollHistorySlot {
    pub slot_id: u32,
    pub die_name: String,
    pub locked: bool,
    pub value: Option<FaceValue>,
}

#[derive(Debug, Clone)]
pub struct StoredState {
    pub engine: DiceEngine,
    pub history: Vec<RollHistoryEntry>,
}

impl StoredState {
    pub fn new(engine: DiceEngine, history: Vec<RollHistoryEntry>) -> Self {
        Self { engine, history }
    }
}

#[derive(Serialize, Deserialize, Default)]
struct StorageData {
    #[serde(default)]
    custom_dice: Vec<Die>,
    #[serde(default)]
    trays: Vec<Tray>,
    #[serde(default)]
    history: Vec<RollHistoryEntry>,
}

pub fn save(path: &Path, engine: &DiceEngine) -> Result<()> {
    save_state(path, engine, &[])
}

pub fn save_state(path: &Path, engine: &DiceEngine, history: &[RollHistoryEntry]) -> Result<()> {
    let data = StorageData {
        custom_dice: engine.custom_dice().into_iter().cloned().collect(),
        trays: engine.trays().to_vec(),
        history: history.to_vec(),
    };

    let serialized =
        toml::to_string_pretty(&data).map_err(|err| DiceError::StorageError(err.to_string()))?;

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| DiceError::StorageError(err.to_string()))?;
    }

    fs::write(path, serialized).map_err(|err| DiceError::StorageError(err.to_string()))?;

    Ok(())
}

pub fn load(path: &Path) -> Result<DiceEngine> {
    Ok(load_state(path)?.engine)
}

pub fn load_state(path: &Path) -> Result<StoredState> {
    let mut engine = DiceEngine::new();
    if !path.exists() {
        return Ok(StoredState::new(engine, Vec::new()));
    }

    let contents =
        fs::read_to_string(path).map_err(|err| DiceError::StorageError(err.to_string()))?;
    let mut data: StorageData =
        toml::from_str(&contents).map_err(|err| DiceError::StorageError(err.to_string()))?;

    for die in &mut data.custom_dice {
        die.kind = DieKind::Custom;
    }

    engine.set_custom_dice(data.custom_dice);
    engine.set_trays(data.trays);

    Ok(StoredState::new(engine, data.history))
}
