//! Error types returned by `rdice_core`.

use thiserror::Error;

/// Errors that can occur while managing dice, trays, expressions, or storage.
#[derive(Debug, Error)]
pub enum DiceError {
    /// The requested tray does not exist.
    #[error("Tray not found: {0}")]
    TrayNotFound(String),
    /// A tray with the requested name already exists.
    #[error("Tray already exists: {0}")]
    TrayAlreadyExists(String),
    /// The requested die does not exist.
    #[error("Die not found: {0}")]
    DieNotFound(String),
    /// A die with the requested canonical name already exists.
    #[error("Die already exists: {0}")]
    DieAlreadyExists(String),
    /// The requested tray slot does not exist.
    #[error("Slot #{slot_id} not found in tray '{tray}'")]
    SlotNotFound {
        /// Tray that was searched.
        tray: String,
        /// Slot identifier that was requested.
        slot_id: u32,
    },
    /// Built-in dice cannot be modified or deleted.
    #[error("Cannot modify builtin die: {0}")]
    CannotModifyBuiltin(String),
    /// The die cannot be deleted because one or more trays reference it.
    #[error("Cannot delete die '{die}' — in use by trays: {trays:?}")]
    CannotDeleteInUse {
        /// Canonical die name that was requested for deletion.
        die: String,
        /// Trays that still reference the die.
        trays: Vec<String>,
    },
    /// Storage serialization or persistence failed.
    #[error("Storage error: {0}")]
    StorageError(String),
    /// A dice expression is malformed or unsupported.
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    /// A command or parser received invalid arguments.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    /// A tray did not contain enough matching dice to satisfy a request.
    #[error(
        "Tray '{tray}' does not contain enough matching dice for '{die}' (requested {requested}, available {available})"
    )]
    InsufficientMatchingDice {
        /// Tray that was searched.
        tray: String,
        /// Die name that was requested.
        die: String,
        /// Number of matching dice requested.
        requested: usize,
        /// Number of matching dice available in the tray.
        available: usize,
    },
    /// A custom die or counted expression had no faces.
    #[error("Die must have at least 1 face")]
    InvalidFaceCount,
    /// Numeric dice must have at least two faces.
    #[error("Numeric dice must have at least 2 faces: {0}")]
    InvalidNumericDie(String),
    /// A name was empty or contained whitespace.
    #[error("Invalid name: must not be empty or contain whitespace")]
    InvalidName,
}

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, DiceError>;
