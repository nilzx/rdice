//! Dice value objects.
//!
//! This module contains the serializable representation of dice and die faces.
//! Built-in dice use numeric faces; custom dice may mix numeric and text faces.

use serde::{Deserialize, Serialize};

/// A single face on a die.
///
/// Integer faces contribute their value to roll sums and point analysis. Text
/// faces render as text and contribute zero points during analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum FaceValue {
    /// A numeric face value.
    #[serde(rename = "integer")]
    Integer(i64),
    /// A text face value.
    #[serde(rename = "text")]
    Text(String),
}

impl std::fmt::Display for FaceValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FaceValue::Integer(n) => write!(f, "{n}"),
            FaceValue::Text(s) => write!(f, "{s}"),
        }
    }
}

/// Classifies whether a die is provided by the crate or supplied by a caller.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum DieKind {
    /// One of the built-in numeric dice.
    Builtin,
    /// A caller-defined die.
    #[default]
    Custom,
}

/// A die with a stable name and ordered set of faces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Die {
    /// Canonical die name.
    ///
    /// Built-in dice are named `D4`, `D6`, `D8`, `D10`, `D12`, `D20`, and
    /// `D100`. Custom dice are stored with [`CUSTOM_PREFIX`].
    pub name: String,
    /// Ordered set of faces that may be rolled.
    pub faces: Vec<FaceValue>,
    /// Whether this die is built-in or custom.
    #[serde(default)]
    pub kind: DieKind,
}

/// Prefix used for canonical custom die names.
pub const CUSTOM_PREFIX: &str = "\u{273d}";

/// Returns the built-in numeric dice.
///
/// The built-in set contains `D4`, `D6`, `D8`, `D10`, `D12`, `D20`, and
/// `D100`.
pub fn builtin_dice() -> Vec<Die> {
    [4, 6, 8, 10, 12, 20, 100]
        .into_iter()
        .map(|n| Die {
            name: format!("D{n}"),
            faces: (1..=n).map(|i| FaceValue::Integer(i as i64)).collect(),
            kind: DieKind::Builtin,
        })
        .collect()
}
