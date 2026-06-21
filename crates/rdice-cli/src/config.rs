use std::fs;
use std::path::Path;

use rdice_core::DiceEngine;
use rdice_core::die::{Die, DieKind, FaceValue};
use rdice_core::error::{DiceError, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    dice: Vec<ConfigDie>,
}

#[derive(Debug, Deserialize)]
struct ConfigDie {
    name: String,
    faces: Vec<ConfigFace>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ConfigFace {
    Integer(i64),
    Text(String),
}

pub fn load_custom_dice(path: &Path, engine: &mut DiceEngine) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let contents =
        fs::read_to_string(path).map_err(|err| DiceError::StorageError(err.to_string()))?;
    let config: ConfigFile =
        toml::from_str(&contents).map_err(|err| DiceError::StorageError(err.to_string()))?;

    let dice = config
        .dice
        .into_iter()
        .map(|die| Die {
            name: die.name,
            faces: die.faces.into_iter().map(Into::into).collect(),
            kind: DieKind::Custom,
        })
        .collect();

    engine.set_custom_dice(dice);
    Ok(())
}

impl From<ConfigFace> for FaceValue {
    fn from(face: ConfigFace) -> Self {
        match face {
            ConfigFace::Integer(value) => FaceValue::Integer(value),
            ConfigFace::Text(value) => FaceValue::Text(value),
        }
    }
}
