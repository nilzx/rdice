//! Stateful dice engine.
//!
//! [`DiceEngine`] combines dice definitions and trays, then exposes the core
//! operations used by user interfaces: dice CRUD, tray CRUD, random rolls, and
//! deterministic roll analysis.

use rand::{Rng, RngExt};

use crate::die::{CUSTOM_PREFIX, Die, DieKind, FaceValue, builtin_dice};
use crate::error::{DiceError, Result};
use crate::tray::{Tray, TraySlot};

/// Result of rolling one die.
#[derive(Debug, Clone, PartialEq)]
pub struct DieRoll {
    /// One-based roll number within a batch, or `1` for a single-die roll.
    pub roll_id: usize,
    /// Canonical name of the die that was rolled.
    pub die_name: String,
    /// Face value selected by the roll.
    pub value: FaceValue,
}

/// Result of rolling multiple dice.
#[derive(Debug, Clone, PartialEq)]
pub struct RollBatchResult {
    /// Individual roll results in input order.
    pub rolls: Vec<DieRoll>,
    /// Sum of integer faces when at least one integer face was rolled.
    ///
    /// Text faces are excluded. This is `None` when the batch contains no
    /// integer face values.
    pub integer_sum: Option<i64>,
}

/// Deterministic analysis for a roll expression.
#[derive(Debug, Clone, PartialEq)]
pub struct RollAnalysis {
    /// Average point value of the roll, including modifiers.
    pub expected_value: f64,
    /// Inclusive minimum and maximum point values, including modifiers.
    pub point_range: PointRange,
}

/// Inclusive integer point range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointRange {
    /// Minimum possible point value.
    pub min: i64,
    /// Maximum possible point value.
    pub max: i64,
}

/// Snapshot result for a tray.
#[derive(Debug, Clone, PartialEq)]
pub struct TrayResult {
    /// Tray name.
    pub tray_name: String,
    /// Current slot snapshots in tray order.
    pub slots: Vec<SlotResult>,
    /// Sum of current integer slot values when any are present.
    ///
    /// Text faces and empty slots are excluded. This is `None` when the tray has
    /// no current integer values.
    pub integer_sum: Option<i64>,
}

/// Snapshot result for one tray slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotResult {
    /// Stable slot identifier within the tray.
    pub slot_id: u32,
    /// Canonical die name assigned to the slot.
    pub die_name: String,
    /// Whether this slot is locked against future tray rolls.
    pub locked: bool,
    /// Most recent rolled value for the slot.
    pub current_value: Option<FaceValue>,
}

/// In-memory dice and tray engine.
///
/// A new engine starts with the built-in dice and no trays. Custom dice and
/// trays are held in memory; callers that need persistence should serialize the
/// data structures exposed by this crate.
#[derive(Debug, Clone)]
pub struct DiceEngine {
    dice: Vec<Die>,
    trays: Vec<Tray>,
}

impl DiceEngine {
    /// Creates a new engine with the built-in dice loaded.
    pub fn new() -> Self {
        Self {
            dice: builtin_dice(),
            trays: Vec::new(),
        }
    }

    /// Replaces all custom dice while preserving the built-in dice.
    ///
    /// Each supplied die is normalized to [`DieKind::Custom`] and receives
    /// [`CUSTOM_PREFIX`] if it does not already have it.
    pub fn set_custom_dice(&mut self, custom: Vec<Die>) {
        let mut dice = builtin_dice();
        dice.extend(custom.into_iter().map(|mut die| {
            let base_name = die.name.strip_prefix(CUSTOM_PREFIX).unwrap_or(&die.name);
            die.name = format!("{CUSTOM_PREFIX}{base_name}");
            die.kind = DieKind::Custom;
            die
        }));
        self.dice = dice;
    }

    /// Replaces the engine's trays.
    pub fn set_trays(&mut self, trays: Vec<Tray>) {
        self.trays = trays;
    }

    /// Returns all custom dice.
    pub fn custom_dice(&self) -> Vec<&Die> {
        self.dice
            .iter()
            .filter(|die| die.kind == DieKind::Custom)
            .collect()
    }

    /// Returns all trays.
    pub fn trays(&self) -> &[Tray] {
        &self.trays
    }

    /// Returns all dice known to the engine.
    ///
    /// Built-in dice are listed before custom dice.
    pub fn list_dice(&self) -> &[Die] {
        &self.dice
    }

    /// Returns all trays known to the engine.
    pub fn list_trays(&self) -> &[Tray] {
        &self.trays
    }

    /// Returns a tray by exact name.
    pub fn get_tray(&self, name: &str) -> Option<&Tray> {
        self.trays.iter().find(|tray| tray.name == name)
    }

    /// Resolves a user-provided die name to its canonical engine name.
    ///
    /// The lookup first tries an exact match, then a case-insensitive match, and
    /// finally the custom-die form with [`CUSTOM_PREFIX`].
    pub fn resolve_die_name(&self, name: &str) -> Option<String> {
        if let Some(die) = self.dice.iter().find(|die| die.name == name) {
            return Some(die.name.clone());
        }

        if let Some(die) = self
            .dice
            .iter()
            .find(|die| die.name.eq_ignore_ascii_case(name))
        {
            return Some(die.name.clone());
        }

        let normalized_name = Self::normalize_custom_name(name).ok()?;
        self.dice
            .iter()
            .find(|die| die.name == normalized_name)
            .map(|die| die.name.clone())
    }

    /// Creates a custom die and returns the stored die.
    ///
    /// The supplied name must be non-empty and contain no whitespace. Names are
    /// normalized with [`CUSTOM_PREFIX`].
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::InvalidName`] for invalid names,
    /// [`DiceError::InvalidFaceCount`] for empty faces, and
    /// [`DiceError::DieAlreadyExists`] when the normalized name is already in
    /// use.
    pub fn create_die(&mut self, name: &str, faces: Vec<FaceValue>) -> Result<&Die> {
        let normalized_name = Self::normalize_custom_name(name)?;
        if faces.is_empty() {
            return Err(DiceError::InvalidFaceCount);
        }
        if self.find_die_index(&normalized_name).is_some() {
            return Err(DiceError::DieAlreadyExists(normalized_name));
        }

        self.dice.push(Die {
            name: normalized_name,
            faces,
            kind: DieKind::Custom,
        });

        Ok(self.dice.last().expect("just pushed custom die"))
    }

    /// Replaces the faces of a custom die and returns the stored die.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::InvalidFaceCount`] for empty faces,
    /// [`DiceError::CannotModifyBuiltin`] for built-in dice,
    /// [`DiceError::InvalidName`] for invalid custom names, and
    /// [`DiceError::DieNotFound`] when the die does not exist.
    pub fn modify_die(&mut self, name: &str, faces: Vec<FaceValue>) -> Result<&Die> {
        if faces.is_empty() {
            return Err(DiceError::InvalidFaceCount);
        }
        if self.is_builtin_name(name) {
            return Err(DiceError::CannotModifyBuiltin(name.to_string()));
        }

        let normalized_name = Self::normalize_custom_name(name)?;
        let die_index = self
            .find_die_index(&normalized_name)
            .ok_or_else(|| DiceError::DieNotFound(normalized_name.clone()))?;

        if self.dice[die_index].kind == DieKind::Builtin {
            return Err(DiceError::CannotModifyBuiltin(
                self.dice[die_index].name.clone(),
            ));
        }

        self.dice[die_index].faces = faces;
        Ok(&self.dice[die_index])
    }

    /// Deletes a custom die.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::CannotModifyBuiltin`] for built-in dice,
    /// [`DiceError::InvalidName`] for invalid custom names,
    /// [`DiceError::DieNotFound`] when the die does not exist, and
    /// [`DiceError::CannotDeleteInUse`] when any tray still references the die.
    pub fn delete_die(&mut self, name: &str) -> Result<()> {
        if self.is_builtin_name(name) {
            return Err(DiceError::CannotModifyBuiltin(name.to_string()));
        }

        let normalized_name = Self::normalize_custom_name(name)?;
        let die_index = self
            .find_die_index(&normalized_name)
            .ok_or_else(|| DiceError::DieNotFound(normalized_name.clone()))?;

        if self.dice[die_index].kind == DieKind::Builtin {
            return Err(DiceError::CannotModifyBuiltin(
                self.dice[die_index].name.clone(),
            ));
        }

        let trays_in_use: Vec<String> = self
            .trays
            .iter()
            .filter(|tray| {
                tray.slots
                    .iter()
                    .any(|slot| slot.die_name == normalized_name)
            })
            .map(|tray| tray.name.clone())
            .collect();

        if !trays_in_use.is_empty() {
            return Err(DiceError::CannotDeleteInUse {
                die: normalized_name,
                trays: trays_in_use,
            });
        }

        self.dice.remove(die_index);
        Ok(())
    }

    /// Creates an empty tray and returns it.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::InvalidName`] for empty names or names containing
    /// whitespace, and [`DiceError::TrayAlreadyExists`] when the name is already
    /// in use.
    pub fn create_tray(&mut self, name: &str) -> Result<&Tray> {
        Self::validate_name(name)?;
        if self.find_tray_index(name).is_some() {
            return Err(DiceError::TrayAlreadyExists(name.to_string()));
        }

        self.trays.push(Tray::new(name.to_string()));
        Ok(self.trays.last().expect("just pushed tray"))
    }

    /// Deletes a tray by exact name.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when the tray does not exist.
    pub fn delete_tray(&mut self, name: &str) -> Result<()> {
        let tray_index = self
            .find_tray_index(name)
            .ok_or_else(|| DiceError::TrayNotFound(name.to_string()))?;
        self.trays.remove(tray_index);
        Ok(())
    }

    /// Renames a tray and returns it.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when `old_name` does not exist,
    /// [`DiceError::InvalidName`] when `new_name` is invalid, and
    /// [`DiceError::TrayAlreadyExists`] when `new_name` belongs to another tray.
    pub fn rename_tray(&mut self, old_name: &str, new_name: &str) -> Result<&Tray> {
        Self::validate_name(new_name)?;
        let tray_index = self
            .find_tray_index(old_name)
            .ok_or_else(|| DiceError::TrayNotFound(old_name.to_string()))?;

        if old_name != new_name && self.find_tray_index(new_name).is_some() {
            return Err(DiceError::TrayAlreadyExists(new_name.to_string()));
        }

        self.trays[tray_index].name = new_name.to_string();
        Ok(&self.trays[tray_index])
    }

    /// Adds a die to a tray and returns the assigned slot identifier.
    ///
    /// The die name is resolved through [`Self::resolve_die_name`]. Slot
    /// identifiers are assigned by the tray and are not reused after removal.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::DieNotFound`] when the die cannot be resolved and
    /// [`DiceError::TrayNotFound`] when the tray does not exist.
    pub fn add_die_to_tray(&mut self, die_name: &str, tray_name: &str) -> Result<u32> {
        let resolved_die_name = self
            .resolve_die_name(die_name)
            .ok_or_else(|| DiceError::DieNotFound(Self::die_not_found_name(die_name)))?;

        let tray_index = self
            .find_tray_index(tray_name)
            .ok_or_else(|| DiceError::TrayNotFound(tray_name.to_string()))?;
        let tray = &mut self.trays[tray_index];

        let slot_id = tray.next_slot_id;
        tray.next_slot_id += 1;
        tray.slots.push(TraySlot {
            die_name: resolved_die_name,
            slot_id,
            locked: false,
            current_value: None,
        });

        Ok(slot_id)
    }

    /// Removes a slot from a tray.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when the tray does not exist and
    /// [`DiceError::SlotNotFound`] when the slot does not exist in that tray.
    pub fn remove_slot(&mut self, tray_name: &str, slot_id: u32) -> Result<()> {
        let tray = self.find_tray_mut(tray_name)?;
        let slot_index = tray
            .slots
            .iter()
            .position(|slot| slot.slot_id == slot_id)
            .ok_or_else(|| DiceError::SlotNotFound {
                tray: tray_name.to_string(),
                slot_id,
            })?;
        tray.slots.remove(slot_index);
        Ok(())
    }

    /// Locks a tray slot so future tray rolls preserve its current value.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when the tray does not exist and
    /// [`DiceError::SlotNotFound`] when the slot does not exist in that tray.
    pub fn lock_slot(&mut self, tray_name: &str, slot_id: u32) -> Result<()> {
        let slot = self.find_slot_mut(tray_name, slot_id)?;
        slot.locked = true;
        Ok(())
    }

    /// Unlocks a tray slot so future tray rolls can update its current value.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when the tray does not exist and
    /// [`DiceError::SlotNotFound`] when the slot does not exist in that tray.
    pub fn unlock_slot(&mut self, tray_name: &str, slot_id: u32) -> Result<()> {
        let slot = self.find_slot_mut(tray_name, slot_id)?;
        slot.locked = false;
        Ok(())
    }

    /// Rolls one die.
    ///
    /// Numeric names such as `d13` are supported even when the die is not stored
    /// in the engine.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::InvalidNumericDie`] for numeric dice with fewer than
    /// two faces and [`DiceError::DieNotFound`] for unresolved named dice.
    pub fn roll_die(&self, die_name: &str) -> Result<DieRoll> {
        let mut rng = rand::rng();
        self.roll_die_with_rng(1, die_name, &mut rng)
    }

    fn roll_die_with_rng(
        &self,
        roll_id: usize,
        die_name: &str,
        rng: &mut impl Rng,
    ) -> Result<DieRoll> {
        let resolved = self.resolve_roll_die_name(die_name)?;

        Ok(DieRoll {
            roll_id,
            die_name: resolved.clone(),
            value: self.roll_face(&resolved, rng)?,
        })
    }

    /// Rolls multiple dice in order.
    ///
    /// The returned roll identifiers start at `1` and follow the input order.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered while resolving or rolling a die.
    pub fn roll_dice(&self, die_names: &[String]) -> Result<RollBatchResult> {
        let mut rolls = Vec::with_capacity(die_names.len());
        let mut integer_sum = 0_i64;
        let mut has_integer = false;
        let mut rng = rand::rng();

        for (index, die_name) in die_names.iter().enumerate() {
            let roll = self.roll_die_with_rng(index + 1, die_name, &mut rng)?;
            if let FaceValue::Integer(value) = &roll.value {
                integer_sum += *value;
                has_integer = true;
            }
            rolls.push(roll);
        }

        Ok(RollBatchResult {
            rolls,
            integer_sum: has_integer.then_some(integer_sum),
        })
    }

    /// Computes expected value and inclusive point range without rolling.
    ///
    /// Integer faces contribute their value. Text faces contribute zero points.
    /// Modifiers are added to both expected value and range.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered while resolving or analyzing a die.
    pub fn analyze_roll(&self, die_names: &[String], modifiers: &[i64]) -> Result<RollAnalysis> {
        let modifier_sum: i64 = modifiers.iter().sum();
        let mut expected_value = modifier_sum as f64;
        let mut min = modifier_sum;
        let mut max = modifier_sum;

        for die_name in die_names {
            let resolved = self.resolve_roll_die_name(die_name)?;
            let die_points = self.analyze_die_points(&resolved)?;
            expected_value += die_points.expected_value;
            min += die_points.point_range.min;
            max += die_points.point_range.max;
        }

        Ok(RollAnalysis {
            expected_value,
            point_range: PointRange { min, max },
        })
    }

    /// Rolls every unlocked slot in a tray and returns the updated tray.
    ///
    /// Locked slots keep their existing `current_value`.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when the tray does not exist and
    /// [`DiceError::DieNotFound`] when a tray slot references a missing die.
    pub fn roll_tray(&mut self, tray_name: &str) -> Result<&Tray> {
        let tray_index = self
            .find_tray_index(tray_name)
            .ok_or_else(|| DiceError::TrayNotFound(tray_name.to_string()))?;

        let slots_to_roll: Vec<(usize, String)> = self.trays[tray_index]
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.locked)
            .map(|(index, slot)| (index, slot.die_name.clone()))
            .collect();

        let mut rng = rand::rng();
        for (slot_index, die_name) in slots_to_roll {
            let value = self.roll_face(&die_name, &mut rng)?;
            self.trays[tray_index].slots[slot_index].current_value = Some(value);
        }

        Ok(&self.trays[tray_index])
    }

    /// Returns a snapshot of a tray and its current slot values.
    ///
    /// # Errors
    ///
    /// Returns [`DiceError::TrayNotFound`] when the tray does not exist.
    pub fn show_tray(&self, tray_name: &str) -> Result<TrayResult> {
        let tray = self
            .trays
            .iter()
            .find(|tray| tray.name == tray_name)
            .ok_or_else(|| DiceError::TrayNotFound(tray_name.to_string()))?;

        let slots: Vec<SlotResult> = tray
            .slots
            .iter()
            .map(|slot| SlotResult {
                slot_id: slot.slot_id,
                die_name: slot.die_name.clone(),
                locked: slot.locked,
                current_value: slot.current_value.clone(),
            })
            .collect();

        let mut sum = 0_i64;
        let mut has_integer = false;
        for slot in &slots {
            if let Some(FaceValue::Integer(value)) = &slot.current_value {
                sum += *value;
                has_integer = true;
            }
        }

        Ok(TrayResult {
            tray_name: tray.name.clone(),
            slots,
            integer_sum: has_integer.then_some(sum),
        })
    }

    fn roll_face(&self, die_name: &str, rng: &mut impl Rng) -> Result<FaceValue> {
        if let Some(face_count) = parse_numeric_die_name(die_name) {
            let value = rng.random_range(1..=face_count);
            return Ok(FaceValue::Integer(value as i64));
        }

        let die_index = self
            .find_die_index(die_name)
            .ok_or_else(|| DiceError::DieNotFound(die_name.to_string()))?;
        let faces = &self.dice[die_index].faces;
        let face_index = rng.random_range(0..faces.len());
        Ok(faces[face_index].clone())
    }

    fn analyze_die_points(&self, die_name: &str) -> Result<RollAnalysis> {
        if let Some(face_count) = parse_numeric_die_name(die_name) {
            if face_count < 2 {
                return Err(DiceError::InvalidNumericDie(die_name.to_string()));
            }
            return Ok(RollAnalysis {
                expected_value: (face_count + 1) as f64 / 2.0,
                point_range: PointRange {
                    min: 1,
                    max: face_count as i64,
                },
            });
        }

        let die_index = self
            .find_die_index(die_name)
            .ok_or_else(|| DiceError::DieNotFound(die_name.to_string()))?;
        let points = self.dice[die_index]
            .faces
            .iter()
            .map(face_point_value)
            .collect::<Vec<_>>();
        let point_sum: i64 = points.iter().sum();
        let min = points.iter().min().copied().unwrap_or(0);
        let max = points.iter().max().copied().unwrap_or(0);

        Ok(RollAnalysis {
            expected_value: point_sum as f64 / points.len() as f64,
            point_range: PointRange { min, max },
        })
    }

    fn resolve_roll_die_name(&self, name: &str) -> Result<String> {
        if let Some(face_count) = parse_numeric_die_name(name) {
            if face_count < 2 {
                return Err(DiceError::InvalidNumericDie(name.to_string()));
            }
            return Ok(format!("D{face_count}"));
        }

        self.resolve_die_name(name)
            .ok_or_else(|| DiceError::DieNotFound(Self::die_not_found_name(name)))
    }

    fn validate_name(name: &str) -> Result<()> {
        if name.trim().is_empty() || name.chars().any(char::is_whitespace) {
            return Err(DiceError::InvalidName);
        }
        Ok(())
    }

    fn normalize_custom_name(name: &str) -> Result<String> {
        let base_name = name.strip_prefix(CUSTOM_PREFIX).unwrap_or(name);
        Self::validate_name(base_name)?;
        Ok(format!("{CUSTOM_PREFIX}{base_name}"))
    }

    fn die_not_found_name(name: &str) -> String {
        if name.starts_with(CUSTOM_PREFIX) || looks_like_builtin_name(name) {
            name.to_string()
        } else {
            format!("{CUSTOM_PREFIX}{name}")
        }
    }

    fn find_die_index(&self, name: &str) -> Option<usize> {
        self.dice.iter().position(|die| die.name == name)
    }

    fn find_tray_index(&self, name: &str) -> Option<usize> {
        self.trays.iter().position(|tray| tray.name == name)
    }

    fn find_tray_mut(&mut self, tray_name: &str) -> Result<&mut Tray> {
        self.trays
            .iter_mut()
            .find(|tray| tray.name == tray_name)
            .ok_or_else(|| DiceError::TrayNotFound(tray_name.to_string()))
    }

    fn find_slot_mut(&mut self, tray_name: &str, slot_id: u32) -> Result<&mut TraySlot> {
        let tray = self.find_tray_mut(tray_name)?;
        tray.slots
            .iter_mut()
            .find(|slot| slot.slot_id == slot_id)
            .ok_or_else(|| DiceError::SlotNotFound {
                tray: tray_name.to_string(),
                slot_id,
            })
    }

    fn is_builtin_name(&self, name: &str) -> bool {
        self.dice
            .iter()
            .any(|die| die.kind == DieKind::Builtin && die.name == name)
    }
}

impl Default for DiceEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn looks_like_builtin_name(name: &str) -> bool {
    name.starts_with('D')
}

fn parse_numeric_die_name(name: &str) -> Option<u64> {
    let face_count = name.strip_prefix('d').or_else(|| name.strip_prefix('D'))?;
    if face_count.is_empty() || !face_count.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    face_count.parse().ok()
}

fn face_point_value(face: &FaceValue) -> i64 {
    match face {
        FaceValue::Integer(value) => *value,
        FaceValue::Text(_) => 0,
    }
}
