use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rdice_core::DiceEngine;
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_tui::app::{App, Screen};
use rdice_tui::command::Command;

fn unique_path(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("rdice-tui-app-{label}-{stamp}.toml"))
}

fn empty_app(label: &str) -> App {
    App::new(DiceEngine::new(), unique_path(label))
}

#[test]
fn app_load_creates_default_tray_when_state_is_missing() {
    let path = unique_path("default-missing");

    let app = App::load_from_path(path.clone()).unwrap();

    assert!(app.engine.get_tray("default").is_some());
    assert!(path.exists());

    let loaded = App::load_from_path(path.clone()).unwrap();
    assert_eq!(loaded.engine.list_trays().len(), 1);
    assert!(loaded.engine.get_tray("default").is_some());
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_creates_tray_and_persists_it() {
    let path = unique_path("tray");
    let mut app = empty_app("tray");
    app.state_path = path.clone();

    app.apply_command(Command::TrayNew("combat".into()))
        .unwrap();

    assert!(app.engine.get_tray("combat").is_some());
    assert!(path.exists());

    let loaded = App::load_from_path(path.clone()).unwrap();
    assert!(loaded.engine.get_tray("combat").is_some());
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_creates_custom_die_from_command_faces() {
    let path = unique_path("die");
    let mut app = empty_app("die");
    app.state_path = path.clone();

    app.apply_command(Command::DiceNew {
        name: "fate".into(),
        faces: vec!["-1".into(), "0".into(), "+".into()],
    })
    .unwrap();

    let die = app.engine.resolve_die_name("fate").unwrap();
    assert_eq!(die, format!("{CUSTOM_PREFIX}fate"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_edits_custom_die_and_renames_tray() {
    let path = unique_path("edit-rename");
    let mut app = empty_app("edit-rename");
    app.state_path = path.clone();

    app.apply_command(Command::DiceNew {
        name: "fate".into(),
        faces: vec!["-1".into(), "0".into(), "+".into()],
    })
    .unwrap();
    app.apply_command(Command::DiceEdit {
        name: "fate".into(),
        faces: vec!["1".into(), "2".into()],
    })
    .unwrap();
    let die_name = app.engine.resolve_die_name("fate").unwrap();
    let die = app
        .engine
        .list_dice()
        .iter()
        .find(|die| die.name == die_name)
        .unwrap();
    assert_eq!(
        die.faces,
        vec![FaceValue::Integer(1), FaceValue::Integer(2)]
    );

    app.engine.create_tray("combat").unwrap();
    app.apply_command(Command::TrayRename {
        old_name: "combat".into(),
        new_name: "battle".into(),
    })
    .unwrap();
    assert!(app.engine.get_tray("combat").is_none());
    assert!(app.engine.get_tray("battle").is_some());
    let _ = std::fs::remove_file(path);
}

#[test]
fn editing_custom_die_clears_current_values_for_referencing_slots() {
    let path = unique_path("edit-clear");
    let mut app = empty_app("edit-clear");
    app.state_path = path.clone();
    app.apply_command(Command::DiceNew {
        name: "fate".into(),
        faces: vec!["1".into(), "2".into()],
    })
    .unwrap();
    app.engine.create_tray("combat").unwrap();
    app.engine.add_die_to_tray("fate", "combat").unwrap();
    app.engine.roll_tray("combat").unwrap();
    assert!(
        app.engine.get_tray("combat").unwrap().slots[0]
            .current_value
            .is_some()
    );

    app.apply_command(Command::DiceEdit {
        name: "fate".into(),
        faces: vec!["+".into()],
    })
    .unwrap();

    assert!(
        app.engine.get_tray("combat").unwrap().slots[0]
            .current_value
            .is_none()
    );
    assert!(
        app.message
            .as_deref()
            .unwrap_or_default()
            .contains("cleared 1 slot")
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_toggles_overview_selection_by_page_id() {
    let path = unique_path("selection");
    let mut app = empty_app("selection");
    app.engine.create_tray("combat").unwrap();
    app.engine.create_tray("loot").unwrap();

    app.toggle_tray_selection(1).unwrap();
    assert_eq!(app.selected_trays, vec!["combat"]);

    app.toggle_tray_selection(1).unwrap();
    assert!(app.selected_trays.is_empty());
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_opens_tray_by_page_id_and_toggles_slot_lock() {
    let path = unique_path("slot");
    let mut app = empty_app("slot");
    app.state_path = path.clone();
    app.engine.create_tray("combat").unwrap();
    app.engine.add_die_to_tray("d6", "combat").unwrap();

    app.open_tray_by_page_id(1).unwrap();
    assert_eq!(app.screen, Screen::TrayDetail("combat".into()));

    app.toggle_slot_lock(1).unwrap();
    assert!(app.engine.get_tray("combat").unwrap().slots[0].locked);
    let _ = std::fs::remove_file(path);
}

#[test]
fn face_parser_keeps_numbers_numeric_and_others_text() {
    assert_eq!(App::parse_face("-2"), FaceValue::Integer(-2));
    assert_eq!(App::parse_face("+1"), FaceValue::Integer(1));
    assert_eq!(App::parse_face("+"), FaceValue::Text("+".into()));
    assert_eq!(App::parse_face("heads"), FaceValue::Text("heads".into()));
}

#[test]
fn add_die_by_page_id_adds_to_current_tray() {
    let path = unique_path("add-die");
    let mut app = empty_app("add-die");
    app.state_path = path.clone();
    app.engine.create_tray("combat").unwrap();
    app.screen = Screen::AddDie("combat".into());

    app.add_die_by_page_id(1).unwrap();

    let tray = app.engine.get_tray("combat").unwrap();
    assert_eq!(tray.slots.len(), 1);
    assert_eq!(tray.slots[0].die_name, "D4");
    let _ = std::fs::remove_file(path);
}

#[test]
fn add_die_by_page_id_uses_add_die_page() {
    let path = unique_path("add-die-page");
    let mut app = empty_app("add-die-page");
    app.state_path = path.clone();
    app.engine
        .create_die("alpha", vec![FaceValue::Integer(1)])
        .unwrap();
    app.engine
        .create_die("beta", vec![FaceValue::Integer(1)])
        .unwrap();
    app.engine
        .create_die("gamma", vec![FaceValue::Integer(1)])
        .unwrap();
    app.engine.create_tray("combat").unwrap();
    app.screen = Screen::AddDie("combat".into());

    app.next_page();
    app.add_die_by_page_id(1).unwrap();

    let tray = app.engine.get_tray("combat").unwrap();
    assert_eq!(tray.slots.len(), 1);
    assert_eq!(tray.slots[0].die_name, format!("{CUSTOM_PREFIX}gamma"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn app_prefills_manager_commands_and_returns_from_add_die() {
    let mut app = empty_app("manager-shortcuts");
    app.engine.create_tray("combat").unwrap();
    app.screen = Screen::Overview;

    app.open_context_manager();
    assert_eq!(app.screen, Screen::TrayManager);
    app.prefill_new_target().unwrap();
    assert_eq!(app.command_buffer, Some("tray new ".into()));

    app.prefill_edit_target(1).unwrap();
    assert_eq!(app.command_buffer, Some("tray rename combat ".into()));

    app.prefill_delete_target(1).unwrap();
    assert_eq!(app.command_buffer, Some("tray delete combat".into()));

    app.screen = Screen::TrayDetail("combat".into());
    app.open_context_manager();
    assert_eq!(app.screen, Screen::DiceManager);
    app.prefill_new_target().unwrap();
    assert_eq!(app.command_buffer, Some("dice new ".into()));

    app.screen = Screen::AddDie("combat".into());
    app.escape();
    assert_eq!(app.screen, Screen::TrayDetail("combat".into()));
}

#[test]
fn manager_prefill_uses_manager_pages() {
    let mut app = empty_app("manager-pages");
    for index in 1..=10 {
        app.engine
            .create_die(&format!("custom{index}"), vec![FaceValue::Integer(1)])
            .unwrap();
        app.engine.create_tray(&format!("tray{index}")).unwrap();
    }

    app.screen = Screen::DiceManager;
    app.next_page();
    app.prefill_edit_target(1).unwrap();
    assert_eq!(app.command_buffer, Some("dice edit custom10 ".into()));

    app.command_buffer = None;
    app.screen = Screen::TrayManager;
    app.next_page();
    app.prefill_delete_target(1).unwrap();
    assert_eq!(app.command_buffer, Some("tray delete tray10".into()));
}

#[test]
fn rolling_trays_records_and_persists_history() {
    let path = unique_path("history");
    let mut app = empty_app("history");
    app.state_path = path.clone();
    app.engine.create_tray("combat").unwrap();
    app.engine.add_die_to_tray("d6", "combat").unwrap();
    app.screen = Screen::TrayDetail("combat".into());

    app.roll_current_tray().unwrap();

    assert_eq!(app.roll_history.len(), 1);
    assert_eq!(app.roll_history[0].tray_name, "combat");
    assert_eq!(app.roll_history[0].slots.len(), 1);

    let loaded = App::load_from_path(path.clone()).unwrap();
    assert_eq!(loaded.roll_history.len(), 1);
    assert_eq!(loaded.roll_history[0].tray_name, "combat");
    let _ = std::fs::remove_file(path);
}

#[test]
fn roll_from_add_die_screen_returns_to_tray_without_rolling() {
    let path = unique_path("add-die-roll");
    let mut app = empty_app("add-die-roll");
    app.state_path = path.clone();
    app.engine.create_tray("combat").unwrap();
    app.engine.add_die_to_tray("d6", "combat").unwrap();
    app.screen = Screen::AddDie("combat".into());

    app.roll_from_current_screen().unwrap();

    assert_eq!(app.screen, Screen::TrayDetail("combat".into()));
    assert!(
        app.engine.get_tray("combat").unwrap().slots[0]
            .current_value
            .is_none()
    );
    assert_eq!(app.message, Some("return to tray before rolling".into()));
    let _ = std::fs::remove_file(path);
}

#[test]
fn history_command_returns_to_previous_screen_once() {
    let mut app = empty_app("history-return");
    app.screen = Screen::TrayDetail("combat".into());

    app.apply_command(Command::History).unwrap();
    assert_eq!(app.screen, Screen::History);

    app.apply_command(Command::History).unwrap();
    app.escape();
    assert_eq!(app.screen, Screen::TrayDetail("combat".into()));
}
