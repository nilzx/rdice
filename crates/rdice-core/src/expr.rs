//! Parsers for compact dice roll expressions.
//!
//! Expressions are provided as already-tokenized command-line style strings,
//! such as `["3d6", "d20", "5"]`. Dice counts are expanded into repeated die
//! names and integer tokens are treated as modifiers where allowed.

use crate::error::{DiceError, Result};

/// Parsed roll input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRoll {
    /// Expanded die names in roll order.
    pub dice: Vec<String>,
    /// Integer modifiers that should be added to roll analysis.
    pub modifiers: Vec<i64>,
}

/// Parses dice expressions and integer modifiers.
///
/// Dice expressions use an optional positive count followed by a die name:
/// `d6`, `3d6`, `D20`, or `2Coin`. Bare integers are returned as modifiers.
///
/// # Errors
///
/// Returns [`DiceError::InvalidArguments`] when `tokens` is empty and
/// [`DiceError::InvalidExpression`] when a token cannot be interpreted as a die
/// expression or integer modifier.
///
/// # Examples
///
/// ```
/// use rdice_core::parse_roll_exprs;
///
/// let parsed = parse_roll_exprs(&["3d6", "Coin", "-2"])?;
///
/// assert_eq!(parsed.dice, vec!["d6", "d6", "d6", "Coin"]);
/// assert_eq!(parsed.modifiers, vec![-2]);
///
/// # Ok::<(), rdice_core::DiceError>(())
/// ```
pub fn parse_roll_exprs(tokens: &[&str]) -> Result<ParsedRoll> {
    if tokens.is_empty() {
        return Err(DiceError::InvalidArguments(
            "expected at least one roll token".to_string(),
        ));
    }

    let mut dice = Vec::new();
    let mut modifiers = Vec::new();

    for token in tokens {
        if let Ok(value) = token.parse::<i64>() {
            modifiers.push(value);
            continue;
        }

        dice.extend(parse_die_token(token)?);
    }

    if dice.is_empty() && modifiers.is_empty() {
        return Err(DiceError::InvalidExpression(
            "expression did not contain dice or modifiers".to_string(),
        ));
    }

    Ok(ParsedRoll { dice, modifiers })
}

/// Parses dice expressions and rejects integer modifiers.
///
/// Use this for commands that accept only dice names or counted dice
/// expressions.
///
/// # Errors
///
/// Returns [`DiceError::InvalidArguments`] when `tokens` is empty,
/// [`DiceError::InvalidExpression`] when a token is a modifier or malformed,
/// and [`DiceError::InvalidFaceCount`] when a counted expression uses `0`.
///
/// # Examples
///
/// ```
/// use rdice_core::parse_dice_only_exprs;
///
/// let dice = parse_dice_only_exprs(&["2d6", "Coin"])?;
///
/// assert_eq!(dice, vec!["d6", "d6", "Coin"]);
///
/// # Ok::<(), rdice_core::DiceError>(())
/// ```
pub fn parse_dice_only_exprs(tokens: &[&str]) -> Result<Vec<String>> {
    if tokens.is_empty() {
        return Err(DiceError::InvalidArguments(
            "expected at least one dice expression".to_string(),
        ));
    }

    let mut dice = Vec::new();
    for token in tokens {
        if token.parse::<i64>().is_ok() {
            return Err(DiceError::InvalidExpression(format!(
                "modifiers are not allowed in dice-only expressions: {token}"
            )));
        }

        dice.extend(parse_die_token(token)?);
    }

    Ok(dice)
}

fn parse_die_token(token: &str) -> Result<Vec<String>> {
    let digit_end = token
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit())
        .map(|(index, _)| index)
        .unwrap_or(token.len());

    if digit_end == token.len() {
        return Err(DiceError::InvalidExpression(format!(
            "expected a die expression like d6 or 2custom, got {token}"
        )));
    }

    let count = if digit_end == 0 {
        1
    } else {
        token[..digit_end]
            .parse::<usize>()
            .map_err(|_| DiceError::InvalidExpression(format!("invalid dice count in {token}")))?
    };

    if count == 0 {
        return Err(DiceError::InvalidFaceCount);
    }

    let die_name = &token[digit_end..];
    if die_name.trim().is_empty() {
        return Err(DiceError::InvalidExpression(format!(
            "missing die name in expression {token}"
        )));
    }

    Ok((0..count).map(|_| die_name.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::{parse_dice_only_exprs, parse_roll_exprs};
    use crate::error::DiceError;

    #[test]
    fn parse_roll_exprs_expands_dice_and_modifiers() {
        let parsed = parse_roll_exprs(&["3d6", "2d20", "d100", "2custom", "5", "-3"]).unwrap();
        assert_eq!(
            parsed.dice,
            vec!["d6", "d6", "d6", "d20", "d20", "d100", "custom", "custom"]
        );
        assert_eq!(parsed.modifiers, vec![5, -3]);
    }

    #[test]
    fn parse_dice_only_rejects_modifiers() {
        let err = parse_dice_only_exprs(&["2d6", "-3"]).unwrap_err();
        assert!(matches!(err, DiceError::InvalidExpression(message) if message.contains("-3")));
    }
}
