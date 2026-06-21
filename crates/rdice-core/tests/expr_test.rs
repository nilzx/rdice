use rdice_core::error::DiceError;
use rdice_core::expr::{parse_dice_only_exprs, parse_roll_exprs};

#[test]
fn parse_roll_exprs_expands_repeated_dice() {
    let parsed = parse_roll_exprs(&["5d6"]).unwrap();
    assert_eq!(parsed.dice, vec!["d6", "d6", "d6", "d6", "d6"]);
    assert!(parsed.modifiers.is_empty());
}

#[test]
fn parse_roll_exprs_supports_mixed_dice_and_modifiers() {
    let parsed = parse_roll_exprs(&["3d6", "2d20", "d100", "2custom", "5", "-3"]).unwrap();
    assert_eq!(
        parsed.dice,
        vec!["d6", "d6", "d6", "d20", "d20", "d100", "custom", "custom"]
    );
    assert_eq!(parsed.modifiers, vec![5, -3]);
}

#[test]
fn parse_dice_only_rejects_modifiers_and_bare_numbers() {
    let err = parse_dice_only_exprs(&["2d6", "+5"]).unwrap_err();
    assert!(matches!(err, DiceError::InvalidExpression(_)));

    let err = parse_dice_only_exprs(&["5"]).unwrap_err();
    assert!(matches!(err, DiceError::InvalidExpression(_)));
}
