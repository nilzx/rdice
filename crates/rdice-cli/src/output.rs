use rdice_core::DiceEngine;
use rdice_core::die::{DieKind, FaceValue};
use rdice_core::engine::{DieRoll, RollAnalysis, RollBatchResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollOutputMode {
    Folded,
    Expanded,
}

pub fn print_dice(engine: &DiceEngine) {
    for die in engine.list_dice() {
        let kind = match die.kind {
            DieKind::Builtin => "builtin",
            DieKind::Custom => "custom",
        };
        let faces = die
            .faces
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        println!("{} ({kind}): [{faces}]", die.name);
    }
}

pub fn print_roll_result(result: &RollBatchResult, modifiers: &[i64], mode: RollOutputMode) {
    match mode {
        RollOutputMode::Folded => print_folded_rolls(&result.rolls),
        RollOutputMode::Expanded => print_expanded_rolls(&result.rolls),
    }

    for modifier in modifiers {
        println!("modifier: {modifier:+}");
    }

    let modifier_sum: i64 = modifiers.iter().sum();
    if should_print_total(result, modifiers, mode) {
        match result.integer_sum {
            Some(sum) => println!("total: {}", sum + modifier_sum),
            None if !modifiers.is_empty() => println!("total: {modifier_sum}"),
            None => {}
        }
    }
}

pub fn print_analysis(analysis: &RollAnalysis, show_expected: bool, show_range: bool) {
    if show_expected {
        println!("expected: {}", analysis.expected_value);
    }
    if show_range {
        println!(
            "range: {}..{}",
            analysis.point_range.min, analysis.point_range.max
        );
    }
}

fn print_expanded_rolls(rolls: &[DieRoll]) {
    for roll in rolls {
        println!("#{} {}: {}", roll.roll_id, roll.die_name, roll.value);
    }
}

fn print_folded_rolls(rolls: &[DieRoll]) {
    let mut groups: Vec<(&str, Vec<&FaceValue>)> = Vec::new();
    for roll in rolls {
        if let Some((_, values)) = groups
            .iter_mut()
            .find(|(die_name, _)| *die_name == roll.die_name)
        {
            values.push(&roll.value);
        } else {
            groups.push((&roll.die_name, vec![&roll.value]));
        }
    }

    for (die_name, values) in groups {
        let shown_values = values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let mut sum = 0_i64;
        let mut has_integer = false;
        for value in values {
            if let FaceValue::Integer(integer) = value {
                sum += integer;
                has_integer = true;
            }
        }
        if has_integer {
            println!("{die_name} x{}: {sum}", shown_values.split(", ").count());
        } else {
            println!("{die_name}: [{shown_values}]");
        }
    }
}

fn should_print_total(result: &RollBatchResult, modifiers: &[i64], mode: RollOutputMode) -> bool {
    let roll_count = result.rolls.len();
    let modifier_count = modifiers.len();
    match mode {
        RollOutputMode::Expanded => roll_count + modifier_count > 1,
        RollOutputMode::Folded => {
            let mut group_names: Vec<&str> = Vec::new();
            for roll in &result.rolls {
                if !group_names.contains(&roll.die_name.as_str()) {
                    group_names.push(&roll.die_name);
                }
            }
            let group_count = group_names.len();
            group_count + modifier_count > 1
        }
    }
}
