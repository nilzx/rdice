use rdice_core::die::FaceValue;
use rdice_tui::screens::history::render_history_text;
use rdice_tui::storage::{RollHistoryEntry, RollHistorySlot};

#[test]
fn history_text_renders_empty_and_recent_rolls() {
    assert_eq!(
        render_history_text(&[]),
        "No rolls yet. Roll a tray to start history."
    );

    let history = vec![RollHistoryEntry {
        tray_name: "combat".into(),
        total: Some(4),
        slots: vec![RollHistorySlot {
            slot_id: 1,
            die_name: "D6".into(),
            locked: true,
            value: Some(FaceValue::Integer(4)),
        }],
    }];

    let text = render_history_text(&history);
    assert!(text.contains("[1] combat total:4"));
    assert!(text.contains("#1 D6=4 lock"));
}
