use rdice_core::error::{DiceError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    ManagerDice,
    ManagerTrays,
    Overview,
    OpenTray(String),
    DiceNew { name: String, faces: Vec<String> },
    DiceDelete(String),
    DiceEdit { name: String, faces: Vec<String> },
    TrayNew(String),
    TrayDelete(String),
    TrayRename { old_name: String, new_name: String },
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
        ["dice", "edit", name, faces @ ..] if !faces.is_empty() => Ok(Command::DiceEdit {
            name: (*name).to_string(),
            faces: faces.iter().map(|face| (*face).to_string()).collect(),
        }),
        ["tray", "new", name] => Ok(Command::TrayNew((*name).to_string())),
        ["tray", "delete", name] => Ok(Command::TrayDelete((*name).to_string())),
        ["tray", "rename", old_name, new_name] => Ok(Command::TrayRename {
            old_name: (*old_name).to_string(),
            new_name: (*new_name).to_string(),
        }),
        ["quit"] | ["q"] => Ok(Command::Quit),
        _ => invalid_command(input),
    }
}

fn invalid_command<T>(input: &str) -> Result<T> {
    Err(DiceError::InvalidArguments(format!(
        "invalid command: {input}"
    )))
}
