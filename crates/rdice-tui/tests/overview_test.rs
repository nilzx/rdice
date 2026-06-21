use rdice_core::DiceEngine;
use rdice_core::die::FaceValue;
use rdice_tui::screens::overview::{
    OverviewOptions, TrayCard, build_tray_cards, format_ev, format_total_line, render_grid_text,
};
use unicode_width::UnicodeWidthStr;

#[test]
fn ev_format_rounds_to_one_decimal() {
    assert_eq!(format_ev(12.04), "12.0");
    assert_eq!(format_ev(12.05), "12.1");
    assert_eq!(format_ev(3.5), "3.5");
}

#[test]
fn total_line_formats_optional_analysis_and_text_marker() {
    assert_eq!(
        format_total_line(Some(8), Some((2, 24)), Some(12.04), true),
        "Total(2-24~12.0):8 +t"
    );
    assert_eq!(
        format_total_line(Some(8), Some((2, 24)), None, false),
        "Total(2-24):8"
    );
    assert_eq!(
        format_total_line(None, None, Some(12.05), false),
        "Total(~12.1):-"
    );
}

#[test]
fn tray_card_groups_numeric_dice_and_marks_hidden_text() {
    let mut engine = DiceEngine::new();
    engine
        .create_die("weather_long_name", vec![FaceValue::Text("rain".into())])
        .unwrap();
    engine.create_tray("travel").unwrap();
    engine.add_die_to_tray("d6", "travel").unwrap();
    engine.add_die_to_tray("d6", "travel").unwrap();
    engine
        .add_die_to_tray("weather_long_name", "travel")
        .unwrap();
    engine.roll_tray("travel").unwrap();

    let cards = build_tray_cards(
        &engine,
        &[],
        0,
        OverviewOptions {
            text_visible: false,
            range_visible: true,
            ev_visible: true,
        },
    )
    .unwrap();

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].page_id, 1);
    assert!(cards[0].composition.contains("2xd6"));
    assert!(cards[0].composition.contains("weather..."));
    assert!(cards[0].total_line.contains("+t"));
}

#[test]
fn overview_grid_renders_only_existing_cards() {
    assert_eq!(
        render_grid_text(&[], 80),
        "No trays on this page. Press m to manage trays."
    );

    let one_card = vec![card(1, "default")];
    let one_grid = render_grid_text(&one_card, 80);
    assert_eq!(one_grid.matches("[1] default").count(), 1);
    assert_eq!(one_grid.matches('|').count(), 8);

    let two_cards = vec![card(1, "default"), card(2, "combat")];
    let two_grid = render_grid_text(&two_cards, 80);
    assert!(two_grid.contains("[1] default"));
    assert!(two_grid.contains("[2] combat"));
    assert_eq!(two_grid.matches('|').count(), 12);

    let four_cards = vec![
        card(1, "default"),
        card(2, "combat"),
        card(3, "loot"),
        card(4, "travel"),
    ];
    let four_grid = render_grid_text(&four_cards, 80);
    assert!(four_grid.contains("[4] travel"));
    assert_eq!(four_grid.matches('|').count(), 24);
}

#[test]
fn overview_grid_keeps_borders_aligned_with_wide_text_faces() {
    let cards = vec![TrayCard {
        page_id: 1,
        name: "中文骰".into(),
        selected: false,
        composition: "天气".into(),
        total_line: "Total:-".into(),
        text_line: Some("Text:晴天".into()),
    }];

    let grid = render_grid_text(&cards, 80);
    let widths = grid.lines().map(UnicodeWidthStr::width).collect::<Vec<_>>();

    assert!(
        widths.windows(2).all(|window| window[0] == window[1]),
        "expected aligned display widths, got {widths:?}\n{grid}"
    );
}

fn card(page_id: usize, name: &str) -> TrayCard {
    TrayCard {
        page_id,
        name: name.into(),
        selected: false,
        composition: "d6".into(),
        total_line: "Total:-".into(),
        text_line: None,
    }
}
