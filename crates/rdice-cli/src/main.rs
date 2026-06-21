mod config;
mod output;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rdice_core::DiceEngine;
use rdice_core::error::{DiceError, Result};
use rdice_core::expr::parse_roll_exprs;

use crate::output::{OutputStyle, RollOutputMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct AnalysisOptions {
    expected: bool,
    range: bool,
}

impl AnalysisOptions {
    fn any(self) -> bool {
        self.expected || self.range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RollArgs {
    mode: RollOutputMode,
    analysis: AnalysisOptions,
    expr: Vec<String>,
}

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let style = OutputStyle::new(output::color_enabled_from(&raw_args));

    if let Err(err) = run(raw_args, style) {
        output::print_error(&err, style);
        std::process::exit(1);
    }
}

fn run(raw_args: Vec<String>, style: OutputStyle) -> Result<()> {
    let args = parse_global_args(raw_args)?;
    let mut engine = DiceEngine::new();
    let config_path = default_config_path()?;

    if matches!(args.as_slice(), [command, subcommand] if command == "config" && subcommand == "edit")
    {
        return edit_config(&config_path);
    }

    config::load_custom_dice(&config_path, &mut engine)?;

    if !starts_with_known_command(&args)
        && let Some((analysis_options, expr)) = parse_top_level_analysis_args(&args)?
    {
        let parsed = parse_expr_args(&expr)?;
        let analysis = engine.analyze_roll(&parsed.dice, &parsed.modifiers)?;
        output::print_analysis(
            &analysis,
            analysis_options.expected,
            analysis_options.range,
            style,
        );
        return Ok(());
    }

    match args.as_slice() {
        [] => {
            print_help();
            Ok(())
        }
        [arg] if arg == "help" || arg == "--help" || arg == "-h" => {
            print_help();
            Ok(())
        }
        [command] if command == "list" => {
            output::print_dice(&engine, style);
            Ok(())
        }
        [command] if command == "config" => {
            println!("{}", config_path.display());
            Ok(())
        }
        [command, subcommand] if command == "config" && subcommand == "path" => {
            println!("{}", config_path.display());
            Ok(())
        }
        [command, args @ ..] if command == "roll" && !args.is_empty() => {
            let roll_args = parse_roll_args(args)?;
            let parsed = parse_expr_args(&roll_args.expr)?;
            let result = engine.roll_dice(&parsed.dice)?;
            output::print_roll_result(&result, &parsed.modifiers, roll_args.mode, style);
            if roll_args.analysis.any() {
                let analysis = engine.analyze_roll(&parsed.dice, &parsed.modifiers)?;
                output::print_analysis(
                    &analysis,
                    roll_args.analysis.expected,
                    roll_args.analysis.range,
                    style,
                );
            }
            Ok(())
        }
        [command, ..] if command == "roll" => Err(DiceError::InvalidArguments(
            "roll requires at least one dice expression".to_string(),
        )),
        [command, ..] => Err(DiceError::InvalidArguments(format!(
            "unknown command: {command}"
        ))),
    }
}

fn parse_global_args(args: Vec<String>) -> Result<Vec<String>> {
    let mut parsed = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--no-color" => {}
            _ => parsed.push(arg),
        }
    }
    Ok(parsed)
}

fn edit_config(config_path: &PathBuf) -> Result<()> {
    ensure_config_file(config_path)?;
    let editor = default_editor()?;
    let status = editor_command(&editor)
        .arg(config_path)
        .status()
        .map_err(|err| {
            DiceError::StorageError(format!("failed to launch editor '{editor}': {err}"))
        })?;

    if !status.success() {
        return Err(DiceError::StorageError(format!(
            "editor '{editor}' exited with status {status}"
        )));
    }

    Ok(())
}

fn ensure_config_file(config_path: &PathBuf) -> Result<()> {
    if let Some(parent) = config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| DiceError::StorageError(err.to_string()))?;
    }

    if !config_path.exists() {
        fs::write(config_path, default_config_template())
            .map_err(|err| DiceError::StorageError(err.to_string()))?;
    }

    Ok(())
}

fn default_editor() -> Result<String> {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .or_else(|_| Ok(default_fallback_editor().to_string()))
}

fn editor_command(editor: &str) -> Command {
    let mut parts = editor.split_whitespace();
    let command = parts.next().expect("editor is never empty");
    let mut editor_command = Command::new(command);
    editor_command.args(parts);
    editor_command
}

fn default_fallback_editor() -> &'static str {
    if cfg!(windows) { "notepad" } else { "vi" }
}

fn default_config_template() -> &'static str {
    r#"# rdice config
#
# Define custom dice here. Numeric dice use [n]d[m] directly in commands,
# so you only need custom dice for text faces or non-standard face lists.
#
# [[dice]]
# name = "coin"
# faces = ["heads", "tails"]
#
# [[dice]]
# name = "fate"
# faces = [-1, 0, 1]
"#
}

fn parse_roll_args(args: &[String]) -> Result<RollArgs> {
    let mut mode = RollOutputMode::Folded;
    let mut analysis = AnalysisOptions::default();
    let mut expr = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-f" | "--folded" => mode = RollOutputMode::Folded,
            "-x" | "--expanded" => mode = RollOutputMode::Expanded,
            "-E" | "--ev" => analysis.expected = true,
            "-R" | "--range" => analysis.range = true,
            value if value.starts_with('-') && value.parse::<i64>().is_err() => {
                return Err(DiceError::InvalidArguments(format!(
                    "unknown roll option: {value}"
                )));
            }
            _ => expr.push(arg.clone()),
        }
    }

    if expr.is_empty() {
        return Err(DiceError::InvalidArguments(
            "roll requires at least one dice expression".to_string(),
        ));
    }

    Ok(RollArgs {
        mode,
        analysis,
        expr,
    })
}

fn parse_top_level_analysis_args(
    args: &[String],
) -> Result<Option<(AnalysisOptions, Vec<String>)>> {
    let mut analysis = AnalysisOptions::default();
    let mut expr = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-E" | "--ev" => analysis.expected = true,
            "-R" | "--range" => analysis.range = true,
            value if value.starts_with('-') && value.parse::<i64>().is_err() => {
                if analysis.any() {
                    return Err(DiceError::InvalidArguments(format!(
                        "unknown analysis option: {value}"
                    )));
                }
                return Ok(None);
            }
            _ => expr.push(arg.clone()),
        }
    }

    if !analysis.any() {
        return Ok(None);
    }
    if expr.is_empty() {
        return Err(DiceError::InvalidArguments(
            "analysis requires at least one dice expression".to_string(),
        ));
    }

    Ok(Some((analysis, expr)))
}

fn starts_with_known_command(args: &[String]) -> bool {
    matches!(
        args.first().map(String::as_str),
        Some("roll" | "list" | "config" | "help" | "--help" | "-h")
    )
}

fn parse_expr_args(expr: &[String]) -> Result<rdice_core::ParsedRoll> {
    if expr.is_empty() {
        return Err(DiceError::InvalidArguments(
            "expected at least one dice expression".to_string(),
        ));
    }

    let expr_refs = expr.iter().map(String::as_str).collect::<Vec<_>>();
    parse_roll_exprs(&expr_refs)
}

fn default_config_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("RDICE_CONFIG_PATH") {
        return Ok(PathBuf::from(path));
    }

    let home = dirs::home_dir().ok_or_else(|| {
        DiceError::StorageError("cannot determine home directory for config path".to_string())
    })?;

    Ok(home.join(".config").join("rdice").join("config.toml"))
}

fn print_help() {
    println!("Usage:");
    println!("  rdice roll [-f|--folded] [-x|--expanded] [-E|--ev] [-R|--range] <dice-expr...>");
    println!("  rdice [-E|--ev] [-R|--range] <dice-expr...>");
    println!("  rdice list");
    println!("  rdice config path");
    println!("  rdice config edit");
    println!("  rdice help [--no-color]");
    println!();
    println!("Options:");
    println!("  --no-color        Disable ANSI color output");
    println!();
    println!("Environment:");
    println!("  RDICE_CONFIG_PATH  Path to rdice config TOML");
    println!("  NO_COLOR           Disable ANSI color output");
}
