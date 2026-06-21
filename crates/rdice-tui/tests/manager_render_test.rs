use rdice_core::DiceEngine;
use rdice_core::die::FaceValue;
use rdice_tui::app::App;
use rdice_tui::screens::{dice_manager, tray};

fn app_with_custom_dice() -> App {
    let mut app = App::new(
        DiceEngine::new(),
        std::env::temp_dir().join("rdice-manager-render-test.toml"),
    );
    for index in 1..=10 {
        app.engine
            .create_die(&format!("custom{index}"), vec![FaceValue::Integer(1)])
            .unwrap();
    }
    app
}

#[test]
fn manager_pages_show_global_and_page_shortcut_numbers() {
    let mut app = app_with_custom_dice();
    app.dice_manager_page = 1;

    let body = dice_manager::render_text(&app);

    assert!(body.contains("[10|1] custom10: 1 faces"));
}

#[test]
fn add_die_pages_show_global_and_page_shortcut_numbers() {
    let mut app = app_with_custom_dice();
    app.add_die_page = 1;

    let body = tray::render_add_die_text(&app);

    assert!(body.contains("[10|1] ✽custom3"));
}
