use ratatui::style::{Color, Modifier, Style};
use ratatui::{Terminal, backend::TestBackend};
use rdice_core::DiceEngine;
use rdice_tui::app::{App, Screen};
use rdice_tui::theme;
use rdice_tui::ui::{footer_text, help_text};

#[test]
fn help_text_changes_by_screen() {
    let mut app = App::new(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-ui-test.toml"),
    );

    app.screen = Screen::Overview;
    assert!(help_text(&app).contains("m trays"));
    assert!(help_text(&app).contains("c dice"));

    app.screen = Screen::TrayDetail("combat".into());
    assert!(help_text(&app).contains("m/c dice"));

    app.screen = Screen::AddDie("combat".into());
    assert!(help_text(&app).contains("Esc tray"));
    assert!(help_text(&app).contains("c dice"));

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
    app.command_buffer = Some("manager dice".into());

    let footer = footer_text(&app, 80);

    assert!(footer.contains("COMMAND: manager dice|"));
    assert!(footer.contains("Enter apply"));
    assert!(footer.contains("Esc cancel"));
}

#[test]
fn footer_explains_dice_creation_wizard_steps() {
    let mut app = App::new(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-ui-test.toml"),
    );
    app.start_dice_creation();

    let footer = footer_text(&app, 80);

    assert_eq!(footer.len(), 80);
    assert!(footer.starts_with("NEW DIE > name: <name>|"));
    assert!(footer.ends_with("Enter next  Esc cancel"));
    assert!(!footer.contains("n new"));
    assert!(!footer.contains(":dice"));

    for ch in "coin".chars() {
        app.push_dice_creation_char(ch);
    }
    let footer = footer_text(&app, 80);
    assert!(footer.contains("NEW DIE > name: coin|"));

    app.advance_dice_creation().unwrap();
    let footer = footer_text(&app, 80);
    assert!(footer.starts_with("NEW DIE > face count: <count>|"));
    assert!(footer.ends_with("Enter next  Esc cancel"));

    app.push_dice_creation_char('2');
    app.advance_dice_creation().unwrap();
    let footer = footer_text(&app, 80);
    assert!(footer.starts_with("NEW DIE > face 1/2: <face>|"));
    assert!(footer.ends_with("Enter next  Esc cancel"));

    for ch in "heads".chars() {
        app.push_dice_creation_char(ch);
    }
    app.advance_dice_creation().unwrap();
    let footer = footer_text(&app, 80);
    assert!(footer.starts_with("NEW DIE > face 2/2: <face>|"));
    assert!(footer.ends_with("Enter create  Esc cancel"));
}

#[test]
fn theme_styles_can_be_disabled_for_no_color_contexts() {
    assert_eq!(theme::border(false), Style::default());
    assert_eq!(theme::content(false), Style::default());
    assert_eq!(theme::footer(false), Style::default());

    let title = theme::title(true);
    assert_eq!(title.fg, Some(Color::Yellow));
    assert!(title.add_modifier.contains(Modifier::BOLD));

    let command_footer = theme::command_footer(true);
    assert_eq!(command_footer.fg, Some(Color::Magenta));
    assert!(command_footer.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn overview_main_content_uses_multiple_colors() {
    let mut app = App::new_with_color(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-ui-color-test.toml"),
        true,
    );
    app.engine.create_tray("combat").unwrap();
    app.engine.add_die_to_tray("d6", "combat").unwrap();
    app.engine.roll_tray("combat").unwrap();
    app.selected_trays.push("combat".into());

    let backend = TestBackend::new(80, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| rdice_tui::ui::render(frame, &app))
        .unwrap();

    let mut colors = Vec::new();
    let buffer = terminal.backend().buffer();
    for y in 0..11 {
        for x in 0..80 {
            let cell = buffer.cell((x, y)).unwrap();
            if cell.symbol() != " " && !colors.contains(&cell.fg) {
                colors.push(cell.fg);
            }
        }
    }

    assert!(
        colors.len() >= 3,
        "expected varied overview colors, got {colors:?}"
    );

    let selected_marker = (0..11).find_map(|y| {
        (0..80).find_map(|x| {
            let cell = buffer.cell((x, y)).unwrap();
            (cell.symbol() == "*").then_some(cell)
        })
    });
    let selected_marker = selected_marker.expect("expected selected tray marker");
    assert_eq!(selected_marker.fg, Color::Red);
    assert!(selected_marker.modifier.contains(Modifier::BOLD));
}
