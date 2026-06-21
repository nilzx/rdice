use rdice_core::die::{CUSTOM_PREFIX, Die, DieKind, FaceValue};
use rdice_core::engine::DiceEngine;
use rdice_core::error::DiceError;
use rdice_core::tray::Tray;

fn engine() -> DiceEngine {
    DiceEngine::new()
}

#[test]
fn new_loads_builtin_dice_and_exposes_state_accessors() {
    let engine = DiceEngine::new();

    assert!(engine.list_trays().is_empty());
    assert_eq!(engine.list_dice().len(), 7);
    assert_eq!(engine.custom_dice().len(), 0);
    assert!(engine.trays().is_empty());
    assert!(engine.list_trays().is_empty());

    let builtin_names: Vec<&str> = engine
        .list_dice()
        .iter()
        .map(|die| die.name.as_str())
        .collect();
    assert_eq!(
        builtin_names,
        vec!["D4", "D6", "D8", "D10", "D12", "D20", "D100"]
    );
    assert!(
        engine
            .list_dice()
            .iter()
            .all(|die| die.kind == DieKind::Builtin)
    );
}

#[test]
fn set_custom_dice_replaces_customs_and_preserves_builtins() {
    let mut engine = engine();

    engine.set_custom_dice(vec![Die {
        name: format!("{CUSTOM_PREFIX}Coin"),
        faces: vec![
            FaceValue::Text("heads".into()),
            FaceValue::Text("tails".into()),
        ],
        kind: DieKind::Builtin,
    }]);

    assert_eq!(engine.list_dice().len(), 8);
    assert_eq!(engine.custom_dice().len(), 1);
    assert_eq!(
        engine.custom_dice()[0].name.as_str(),
        format!("{CUSTOM_PREFIX}Coin").as_str()
    );
    assert_eq!(engine.custom_dice()[0].kind, DieKind::Custom);
    assert_eq!(engine.list_dice()[0].name.as_str(), "D4");

    engine.set_custom_dice(Vec::new());

    assert_eq!(engine.list_dice().len(), 7);
    assert!(engine.custom_dice().is_empty());
}

#[test]
fn create_and_modify_custom_die_validate_inputs_and_prefix_names() {
    let mut engine = engine();

    let err = engine
        .create_die("", vec![FaceValue::Integer(1)])
        .unwrap_err();
    assert!(matches!(err, DiceError::InvalidName));

    let err = engine
        .create_die("bad name", vec![FaceValue::Integer(1)])
        .unwrap_err();
    assert!(matches!(err, DiceError::InvalidName));

    let err = engine.create_die("Coin", Vec::new()).unwrap_err();
    assert!(matches!(err, DiceError::InvalidFaceCount));

    let die = engine
        .create_die("Coin", vec![FaceValue::Text("heads".into())])
        .unwrap();
    assert_eq!(die.name.as_str(), format!("{CUSTOM_PREFIX}Coin").as_str());
    assert_eq!(die.kind, DieKind::Custom);

    let err = engine
        .create_die("Coin", vec![FaceValue::Text("tails".into())])
        .unwrap_err();
    assert!(matches!(
        err,
        DiceError::DieAlreadyExists(name) if name == format!("{CUSTOM_PREFIX}Coin")
    ));

    let die = engine
        .modify_die("Coin", vec![FaceValue::Integer(1), FaceValue::Integer(2)])
        .unwrap();
    assert_eq!(
        die.faces,
        vec![FaceValue::Integer(1), FaceValue::Integer(2)]
    );

    let err = engine.modify_die("Coin", Vec::new()).unwrap_err();
    assert!(matches!(err, DiceError::InvalidFaceCount));

    let err = engine
        .modify_die("D6", vec![FaceValue::Integer(1)])
        .unwrap_err();
    assert!(matches!(
        err,
        DiceError::CannotModifyBuiltin(name) if name == "D6"
    ));

    let err = engine
        .modify_die("Missing", vec![FaceValue::Integer(1)])
        .unwrap_err();
    assert!(matches!(
        err,
        DiceError::DieNotFound(name) if name == format!("{CUSTOM_PREFIX}Missing")
    ));
}

#[test]
fn delete_die_rejects_builtin_and_in_use_then_removes_custom() {
    let mut engine = engine();
    engine
        .create_die("Token", vec![FaceValue::Integer(9)])
        .unwrap();
    engine.create_tray("bag").unwrap();
    let slot_id = engine.add_die_to_tray("Token", "bag").unwrap();

    let err = engine.delete_die("D6").unwrap_err();
    assert!(matches!(
        err,
        DiceError::CannotModifyBuiltin(name) if name == "D6"
    ));

    let err = engine.delete_die("Token").unwrap_err();
    assert!(matches!(
        err,
        DiceError::CannotDeleteInUse { die, trays }
            if die == format!("{CUSTOM_PREFIX}Token") && trays == vec!["bag".to_string()]
    ));

    engine.remove_slot("bag", slot_id).unwrap();
    engine.delete_die("Token").unwrap();

    assert!(engine.custom_dice().is_empty());

    let err = engine.delete_die("Token").unwrap_err();
    assert!(matches!(
        err,
        DiceError::DieNotFound(name) if name == format!("{CUSTOM_PREFIX}Token")
    ));
}

#[test]
fn create_delete_and_replace_trays_cover_tray_accessors() {
    let mut engine = engine();

    let err = engine.create_tray("").unwrap_err();
    assert!(matches!(err, DiceError::InvalidName));

    let err = engine.create_tray("bad tray").unwrap_err();
    assert!(matches!(err, DiceError::InvalidName));

    engine.create_tray("main").unwrap();
    assert_eq!(engine.trays().len(), 1);
    assert_eq!(engine.list_trays()[0].name.as_str(), "main");

    engine.rename_tray("main", "renamed").unwrap();
    assert!(engine.get_tray("main").is_none());
    assert_eq!(engine.list_trays()[0].name.as_str(), "renamed");

    let err = engine.rename_tray("renamed", "bad tray").unwrap_err();
    assert!(matches!(err, DiceError::InvalidName));

    engine.create_tray("other").unwrap();
    let err = engine.rename_tray("renamed", "other").unwrap_err();
    assert!(matches!(
        err,
        DiceError::TrayAlreadyExists(name) if name == "other"
    ));

    let err = engine.create_tray("renamed").unwrap_err();
    assert!(matches!(
        err,
        DiceError::TrayAlreadyExists(name) if name == "renamed"
    ));

    engine.set_trays(vec![Tray::new("loaded".into())]);
    assert_eq!(engine.trays().len(), 1);
    assert_eq!(engine.trays()[0].name.as_str(), "loaded");

    engine.delete_tray("loaded").unwrap();
    assert!(engine.trays().is_empty());

    let err = engine.delete_tray("missing").unwrap_err();
    assert!(matches!(
        err,
        DiceError::TrayNotFound(name) if name == "missing"
    ));
}

#[test]
fn slot_management_roll_lock_and_unlock_behave_consistently() {
    let mut engine = engine();
    engine
        .create_die("One", vec![FaceValue::Integer(1)])
        .unwrap();
    engine
        .create_die("Word", vec![FaceValue::Text("hi".into())])
        .unwrap();
    engine.create_tray("play").unwrap();

    let first_slot = engine.add_die_to_tray("One", "play").unwrap();
    let second_slot = engine
        .add_die_to_tray(&format!("{CUSTOM_PREFIX}Word"), "play")
        .unwrap();
    assert_eq!(first_slot, 1);
    assert_eq!(second_slot, 2);

    let err = engine.add_die_to_tray("Missing", "play").unwrap_err();
    assert!(matches!(
        err,
        DiceError::DieNotFound(name) if name == format!("{CUSTOM_PREFIX}Missing")
    ));

    let err = engine.add_die_to_tray("One", "missing").unwrap_err();
    assert!(matches!(
        err,
        DiceError::TrayNotFound(name) if name == "missing"
    ));

    engine.roll_tray("play").unwrap();
    let shown = engine.show_tray("play").unwrap();
    assert_eq!(shown.tray_name, "play");
    assert_eq!(shown.integer_sum, Some(1));
    assert_eq!(shown.slots.len(), 2);
    assert_eq!(shown.slots[0].slot_id, first_slot);
    assert_eq!(shown.slots[0].current_value, Some(FaceValue::Integer(1)));
    assert_eq!(shown.slots[1].slot_id, second_slot);
    assert_eq!(
        shown.slots[1].current_value,
        Some(FaceValue::Text("hi".into()))
    );

    engine.lock_slot("play", first_slot).unwrap();
    engine
        .modify_die("One", vec![FaceValue::Integer(99)])
        .unwrap();
    engine.roll_tray("play").unwrap();

    let locked_result = engine.show_tray("play").unwrap();
    assert!(locked_result.slots[0].locked);
    assert_eq!(
        locked_result.slots[0].current_value,
        Some(FaceValue::Integer(1))
    );
    assert_eq!(locked_result.integer_sum, Some(1));

    engine.unlock_slot("play", first_slot).unwrap();
    engine.roll_tray("play").unwrap();

    let unlocked_result = engine.show_tray("play").unwrap();
    assert!(!unlocked_result.slots[0].locked);
    assert_eq!(
        unlocked_result.slots[0].current_value,
        Some(FaceValue::Integer(99))
    );
    assert_eq!(unlocked_result.integer_sum, Some(99));

    engine.remove_slot("play", second_slot).unwrap();
    let remaining = engine.show_tray("play").unwrap();
    assert_eq!(remaining.slots.len(), 1);

    let third_slot = engine.add_die_to_tray("One", "play").unwrap();
    assert_eq!(third_slot, 3);

    let err = engine.remove_slot("play", 999).unwrap_err();
    assert!(matches!(
        err,
        DiceError::SlotNotFound { tray, slot_id } if tray == "play" && slot_id == 999
    ));

    let err = engine.lock_slot("play", 999).unwrap_err();
    assert!(matches!(
        err,
        DiceError::SlotNotFound { tray, slot_id } if tray == "play" && slot_id == 999
    ));

    let err = engine.unlock_slot("play", 999).unwrap_err();
    assert!(matches!(
        err,
        DiceError::SlotNotFound { tray, slot_id } if tray == "play" && slot_id == 999
    ));

    let err = engine.roll_tray("missing").unwrap_err();
    assert!(matches!(
        err,
        DiceError::TrayNotFound(name) if name == "missing"
    ));
}

#[test]
fn show_tray_returns_none_sum_when_only_text_faces_are_present() {
    let mut engine = engine();
    engine
        .create_die("Phrase", vec![FaceValue::Text("zap".into())])
        .unwrap();
    engine.create_tray("words").unwrap();
    engine.add_die_to_tray("Phrase", "words").unwrap();

    engine.roll_tray("words").unwrap();
    let tray = engine.show_tray("words").unwrap();

    assert_eq!(tray.integer_sum, None);
    assert_eq!(tray.slots.len(), 1);
    assert_eq!(
        tray.slots[0].current_value,
        Some(FaceValue::Text("zap".into()))
    );

    let err = engine.show_tray("missing").unwrap_err();
    assert!(matches!(
        err,
        DiceError::TrayNotFound(name) if name == "missing"
    ));
}

#[test]
fn roll_die_and_roll_dice_use_basic_name_based_api() {
    let mut engine = engine();
    engine
        .create_die("custom", vec![FaceValue::Integer(7)])
        .unwrap();

    let single = engine.roll_die("d6").unwrap();
    assert_eq!(single.roll_id, 1);
    assert_eq!(single.die_name, "D6");
    assert!(matches!(single.value, FaceValue::Integer(value) if (1..=6).contains(&value)));

    let dynamic = engine.roll_die("d13").unwrap();
    assert_eq!(dynamic.die_name, "D13");
    assert!(matches!(dynamic.value, FaceValue::Integer(value) if (1..=13).contains(&value)));

    let err = engine.roll_die("d1").unwrap_err();
    assert!(matches!(err, DiceError::InvalidNumericDie(name) if name == "d1"));

    let batch = engine
        .roll_dice(&[
            "d6".to_string(),
            "D20".to_string(),
            "custom".to_string(),
            "custom".to_string(),
        ])
        .unwrap();

    assert_eq!(batch.rolls.len(), 4);
    assert_eq!(batch.rolls[0].roll_id, 1);
    assert_eq!(batch.rolls[1].roll_id, 2);
    assert_eq!(batch.rolls[2].roll_id, 3);
    assert_eq!(batch.rolls[3].roll_id, 4);
    assert_eq!(batch.rolls[0].die_name, "D6");
    assert_eq!(batch.rolls[1].die_name, "D20");
    assert_eq!(batch.rolls[2].die_name, format!("{CUSTOM_PREFIX}custom"));
    assert_eq!(batch.rolls[3].die_name, format!("{CUSTOM_PREFIX}custom"));
    assert_eq!(batch.rolls[2].value, FaceValue::Integer(7));
    assert_eq!(batch.rolls[3].value, FaceValue::Integer(7));
    assert!(batch.integer_sum.is_some());
}

#[test]
fn analyze_roll_counts_numeric_dice_and_modifiers() {
    let engine = engine();
    let analysis = engine
        .analyze_roll(
            &["d6".to_string(), "d6".to_string(), "d6".to_string()],
            &[5, -3],
        )
        .unwrap();

    assert_eq!(analysis.expected_value, 12.5);
    assert_eq!(analysis.point_range.min, 5);
    assert_eq!(analysis.point_range.max, 20);
}

#[test]
fn analyze_roll_treats_text_faces_as_zero_points() {
    let mut engine = engine();
    engine
        .create_die(
            "mixed",
            vec![
                FaceValue::Integer(1),
                FaceValue::Text("miss".into()),
                FaceValue::Integer(3),
            ],
        )
        .unwrap();

    let analysis = engine
        .analyze_roll(&["mixed".to_string(), "mixed".to_string()], &[])
        .unwrap();

    assert!((analysis.expected_value - (8.0 / 3.0)).abs() < f64::EPSILON);
    assert_eq!(analysis.point_range.min, 0);
    assert_eq!(analysis.point_range.max, 6);
}

#[test]
fn analyze_roll_rejects_invalid_numeric_dice() {
    let engine = engine();
    let err = engine.analyze_roll(&["d1".to_string()], &[]).unwrap_err();

    assert!(matches!(err, DiceError::InvalidNumericDie(name) if name == "d1"));
}
