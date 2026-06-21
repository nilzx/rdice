use rdice_core::DiceEngine;
use rdice_tui::app::{App, Screen};
use rdice_tui::ui::{footer_text, help_text};

#[test]
fn help_text_changes_by_screen() {
    let mut app = App::new(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-ui-test.toml"),
    );

    app.screen = Screen::Overview;
    assert!(help_text(&app).contains("m trays"));

    app.screen = Screen::TrayDetail("combat".into());
    assert!(help_text(&app).contains("m dice"));

    app.screen = Screen::AddDie("combat".into());
    assert!(help_text(&app).contains("Esc tray"));

    app.screen = Screen::TrayManager;
    assert!(help_text(&app).contains("n new"));

    app.screen = Screen::DiceManager;
    assert!(help_text(&app).contains("e<num> edit"));
}

#[test]
fn footer_prioritizes_feedback_on_narrow_width() {
    let mut app = App::new(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-ui-test.toml"),
    );
    app.message = Some("added D6 to combat".into());

    let footer = footer_text(&app, 10);

    assert_eq!(footer, "added D6 t");
}

#[test]
fn footer_labels_command_mode_with_actions() {
    let mut app = App::new(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-ui-test.toml"),
    );
    app.command_buffer = Some("dice edit c10 ".into());

    let footer = footer_text(&app, 80);

    assert!(footer.contains("COMMAND: dice edit c10 "));
    assert!(footer.contains("Enter apply"));
    assert!(footer.contains("Esc cancel"));
}
