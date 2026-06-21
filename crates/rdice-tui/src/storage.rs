use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use rdice_core::die::{Die, DieKind};
use rdice_core::engine::DiceEngine;
use rdice_core::error::{DiceError, Result};
use rdice_core::tray::Tray;

#[derive(Serialize, Deserialize, Default)]
struct StorageData {
    #[serde(default)]
    custom_dice: Vec<Die>,
    #[serde(default)]
    trays: Vec<Tray>,
}

pub fn save(path: &Path, engine: &DiceEngine) -> Result<()> {
    let data = StorageData {
        custom_dice: engine.custom_dice().into_iter().cloned().collect(),
        trays: engine.trays().to_vec(),
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
    let mut engine = DiceEngine::new();
    if !path.exists() {
        return Ok(engine);
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

    Ok(engine)
}
