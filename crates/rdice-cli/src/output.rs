use rdice_core::DiceEngine;
use rdice_core::die::{DieKind, FaceValue};
use rdice_core::engine::{DieRoll, RollAnalysis, RollBatchResult};

const RESET: &str = "\x1b[0m";
const BOLD_GREEN: &str = "\x1b[1;32m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollOutputMode {
    Folded,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputStyle {
    color: bool,
}

impl OutputStyle {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    pub fn color_enabled(self) -> bool {
        self.color
    }
}

pub fn color_enabled_from(args: &[String]) -> bool {
    std::env::var_os("NO_COLOR").is_none() && !args.iter().any(|arg| arg == "--no-color")
}

pub fn print_error(err: &impl std::fmt::Display, style: OutputStyle) {
    eprintln!("{}: {err}", paint(style, RED, "Error"));
}

pub fn print_dice(engine: &DiceEngine, style: OutputStyle) {
    for die in engine.list_dice() {
        let kind = match die.kind {
            DieKind::Builtin => "builtin",
            DieKind::Custom => "custom",
        };
        let faces = format_die_faces(die.kind, &die.faces);
        let line = format!("{} ({kind}): [{faces}]", die.name);
        let color = match die.kind {
            DieKind::Builtin => DIM,
            DieKind::Custom => CYAN,
        };
        println!("{}", paint(style, color, line));
    }
}

fn format_die_faces(kind: DieKind, faces: &[FaceValue]) -> String {
    if kind == DieKind::Builtin
        && faces.len() > 20
        && faces.iter().enumerate().all(
            |(index, face)| matches!(face, FaceValue::Integer(value) if *value == index as i64 + 1),
        )
    {
        return format!("1..{}", faces.len());
    }

    faces
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn print_roll_result(
    result: &RollBatchResult,
    modifiers: &[i64],
    mode: RollOutputMode,
    style: OutputStyle,
) {
    match mode {
        RollOutputMode::Folded => print_folded_rolls(&result.rolls, style),
        RollOutputMode::Expanded => print_expanded_rolls(&result.rolls, style),
    }

    for modifier in modifiers {
        println!(
            "{}",
            paint(style, YELLOW, format!("modifier: {modifier:+}"))
        );
    }

    let modifier_sum: i64 = modifiers.iter().sum();
    if should_print_total(result, modifiers, mode) {
        match result.integer_sum {
            Some(sum) => println!(
                "{}",
                paint(style, BOLD_GREEN, format!("total: {}", sum + modifier_sum))
            ),
            None if !modifiers.is_empty() => {
                println!(
                    "{}",
                    paint(style, BOLD_GREEN, format!("total: {modifier_sum}"))
                );
            }
            None => {}
        }
    }
}

pub fn print_analysis(
    analysis: &RollAnalysis,
    show_expected: bool,
    show_range: bool,
    style: OutputStyle,
) {
    if show_expected {
        println!(
            "{}",
            paint(
                style,
                MAGENTA,
                format!("expected: {}", analysis.expected_value)
            )
        );
    }
    if show_range {
        let line = format!(
            "range: {}..{}",
            analysis.point_range.min, analysis.point_range.max
        );
        println!("{}", paint(style, MAGENTA, line));
    }
}

fn print_expanded_rolls(rolls: &[DieRoll], style: OutputStyle) {
    for roll in rolls {
        let line = format!("#{} {}: {}", roll.roll_id, roll.die_name, roll.value);
        println!("{}", paint(style, CYAN, line));
    }
}

fn print_folded_rolls(rolls: &[DieRoll], style: OutputStyle) {
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
            let line = format!("{die_name} x{}: {sum}", shown_values.split(", ").count());
            println!("{}", paint(style, CYAN, line));
        } else {
            let line = format!("{die_name}: [{shown_values}]");
            println!("{}", paint(style, CYAN, line));
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

fn paint(style: OutputStyle, ansi: &str, text: impl std::fmt::Display) -> String {
    if style.color_enabled() {
        format!("{ansi}{text}{RESET}")
    } else {
        text.to_string()
    }
}
