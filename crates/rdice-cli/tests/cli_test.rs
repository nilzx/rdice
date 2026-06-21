use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin_path() -> &'static str {
    env!("CARGO_BIN_EXE_rdice")
}

fn unique_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rdice-{label}-{stamp}.toml"))
}

fn run_cli(config_path: &PathBuf, args: &[&str]) -> std::process::Output {
    Command::new(bin_path())
        .env("RDICE_CONFIG_PATH", config_path)
        .args(args)
        .output()
        .expect("failed to run rdice")
}

fn run_cli_with_editor(config_path: &PathBuf, editor: &str, args: &[&str]) -> std::process::Output {
    Command::new(bin_path())
        .env("RDICE_CONFIG_PATH", config_path)
        .env("EDITOR", editor)
        .env_remove("VISUAL")
        .args(args)
        .output()
        .expect("failed to run rdice")
}

#[test]
fn cli_without_args_shows_help() {
    let path = unique_path("help");
    let output = run_cli(&path, &[]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains(
        "rdice roll [-f|--folded] [-x|--expanded] [-E|--ev] [-R|--range] <dice-expr...>"
    ));
    assert!(stdout.contains("rdice [-E|--ev] [-R|--range] <dice-expr...>"));
}

#[test]
fn cli_config_path_uses_explicit_environment_variable() {
    let path = unique_path("config-path");
    let output = run_cli(&path, &["config", "path"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), path.display().to_string());
}

#[test]
fn cli_config_edit_creates_config_and_runs_editor() {
    let path = unique_path("config-edit");
    let output = run_cli_with_editor(&path, "true", &["config", "edit"]);
    assert!(output.status.success());
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("# rdice config"));
    assert!(contents.contains("[[dice]]"));
}

#[test]
fn cli_roll_parses_mixed_expressions() {
    let path = unique_path("expr-roll");
    let output = run_cli(&path, &["roll", "3d6", "2d20", "5", "-3"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("D6 x3:"));
    assert!(stdout.contains("D20 x2:"));
    assert!(stdout.contains("modifier: +5"));
    assert!(stdout.contains("modifier: -3"));
    assert!(stdout.contains("total:"));
}

#[test]
fn cli_roll_supports_expanded_numbered_output_and_dynamic_numeric_dice() {
    let path = unique_path("expanded-roll");
    let output = run_cli(&path, &["roll", "--expanded", "4d13"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#1 D13:"));
    assert!(stdout.contains("#2 D13:"));
    assert!(stdout.contains("#3 D13:"));
    assert!(stdout.contains("#4 D13:"));
    assert!(stdout.contains("total:"));
}

#[test]
fn cli_roll_supports_expanded_short_flag() {
    let path = unique_path("expanded-short-roll");
    let output = run_cli(&path, &["roll", "-x", "2d13"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#1 D13:"));
    assert!(stdout.contains("#2 D13:"));
    assert!(stdout.contains("total:"));
}

#[test]
fn cli_roll_can_append_expected_value_and_range() {
    let path = unique_path("roll-analysis");
    let output = run_cli(&path, &["roll", "-E", "-R", "3d6", "5", "-3"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("D6 x3:"));
    assert!(stdout.contains("modifier: +5"));
    assert!(stdout.contains("modifier: -3"));
    assert!(stdout.contains("expected: 12.5"));
    assert!(stdout.contains("range: 5..20"));
}

#[test]
fn cli_top_level_analysis_flags_do_not_roll() {
    let path = unique_path("top-level-analysis");
    let output = run_cli(&path, &["-E", "-R", "3d6", "5", "-3"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("expected: 12.5"));
    assert!(stdout.contains("range: 5..20"));
    assert!(!stdout.contains("D6 x3:"));
    assert!(!stdout.contains("modifier:"));
}

#[test]
fn cli_folded_single_group_outputs_only_group_sum() {
    let path = unique_path("folded-roll");
    let output = run_cli(&path, &["roll", "4d6"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("D6 x4:"));
    assert!(!stdout.contains("total:"));
    assert!(!stdout.contains("#1 D6:"));
}

#[test]
fn cli_folded_output_preserves_first_seen_die_order() {
    let path = unique_path("folded-order");
    let output = run_cli(&path, &["roll", "2d20", "3d6", "d20"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let d20_index = stdout.find("D20 x3:").unwrap();
    let d6_index = stdout.find("D6 x3:").unwrap();
    assert!(d20_index < d6_index);
}

#[test]
fn cli_roll_rejects_numeric_dice_with_less_than_two_faces() {
    let path = unique_path("invalid-numeric-die");
    let output = run_cli(&path, &["roll", "d1"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Numeric dice must have at least 2 faces"));
}

#[test]
fn cli_loads_custom_dice_from_config() {
    let path = unique_path("custom-config");
    fs::write(
        &path,
        r#"
[[dice]]
name = "coin"
faces = ["heads", "tails"]

[[dice]]
name = "fate"
faces = [-1, 0, 1]
"#,
    )
    .unwrap();

    let list = run_cli(&path, &["list"]);
    assert!(list.status.success());
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("✽coin (custom): [heads, tails]"));
    assert!(stdout.contains("✽fate (custom): [-1, 0, 1]"));

    let roll = run_cli(&path, &["roll", "coin", "2fate"]);
    assert!(roll.status.success());
    let stdout = String::from_utf8_lossy(&roll.stdout);
    assert!(stdout.contains("✽coin:"));
    assert!(stdout.contains("✽fate x2:"));
}

#[test]
fn cli_rejects_removed_legacy_commands() {
    let path = unique_path("legacy");
    let output = run_cli(&path, &["repl"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown command: repl"));
}
