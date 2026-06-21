#![warn(missing_docs)]
//! Core dice primitives and roll orchestration for `rdice`.
//!
//! `rdice_core` owns the domain model used by the command-line and terminal
//! interfaces: dice, trays, roll expressions, random rolls, and deterministic
//! roll analysis.
//!
//! # Quick start
//!
//! ```
//! use rdice_core::{DiceEngine, FaceValue, parse_roll_exprs};
//!
//! let mut engine = DiceEngine::new();
//! engine.create_die(
//!     "Coin",
//!     vec![FaceValue::Text("heads".into()), FaceValue::Text("tails".into())],
//! )?;
//!
//! let parsed = parse_roll_exprs(&["2d6", "Coin", "3"])?;
//! let analysis = engine.analyze_roll(&parsed.dice, &parsed.modifiers)?;
//!
//! assert_eq!(analysis.point_range.min, 5);
//! assert_eq!(analysis.point_range.max, 15);
//!
//! # Ok::<(), rdice_core::DiceError>(())
//! ```
//!
//! # Naming
//!
//! Built-in dice are exposed with uppercase names such as `D6` and `D20`.
//! Numeric roll expressions are case-insensitive, so `d6` and `D6` resolve to
//! the same die. Custom dice are stored with [`CUSTOM_PREFIX`] internally, but
//! API methods that accept die names also accept the unprefixed custom name.

/// Dice definitions and face values.
pub mod die;
/// Roll execution, roll analysis, and tray operations.
pub mod engine;
/// Error and result types returned by the crate.
pub mod error;
/// Parsing helpers for compact dice roll expressions.
pub mod expr;
/// Tray data structures used to group persistent dice slots.
pub mod tray;

pub use die::{CUSTOM_PREFIX, Die, DieKind, FaceValue, builtin_dice};
pub use engine::{
    DiceEngine, DieRoll, PointRange, RollAnalysis, RollBatchResult, SlotResult, TrayResult,
};
pub use error::{DiceError, Result};
pub use expr::{ParsedRoll, parse_dice_only_exprs, parse_roll_exprs};
pub use tray::{Tray, TraySlot};
