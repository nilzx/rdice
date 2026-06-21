//! Persistent dice tray data structures.
//!
//! A tray groups dice into slots so callers can roll the group repeatedly while
//! locking selected slots between rolls.

use super::die::FaceValue;
use serde::{Deserialize, Serialize};

/// A single die slot inside a [`Tray`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraySlot {
    /// Canonical name of the die assigned to this slot.
    pub die_name: String,
    /// Stable slot identifier within the tray.
    pub slot_id: u32,
    /// Whether rolling the tray should preserve this slot's current value.
    pub locked: bool,
    /// Most recent rolled value for this slot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<FaceValue>,
}

/// A named collection of dice slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tray {
    /// Tray name.
    pub name: String,
    /// Slots currently assigned to the tray.
    #[serde(default)]
    pub slots: Vec<TraySlot>,
    /// Next slot identifier to assign.
    ///
    /// Slot identifiers are auto-incrementing and are not reused after a slot is
    /// removed from the tray.
    #[serde(default = "default_next_slot_id")]
    pub next_slot_id: u32,
}

fn default_next_slot_id() -> u32 {
    1
}

impl Tray {
    /// Creates an empty tray with slot identifiers starting at `1`.
    pub fn new(name: String) -> Self {
        Self {
            name,
            slots: Vec::new(),
            next_slot_id: 1,
        }
    }
}
