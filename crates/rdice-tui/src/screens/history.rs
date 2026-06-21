use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::storage::RollHistoryEntry;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(
        Paragraph::new(render_history_text(&app.roll_history)).block(
            Block::default()
                .title(" Roll History ")
                .borders(Borders::ALL),
        ),
        area,
    );
}

pub fn render_history_text(history: &[RollHistoryEntry]) -> String {
    if history.is_empty() {
        return "No rolls yet. Roll a tray to start history.".to_string();
    }

    history
        .iter()
        .take(20)
        .enumerate()
        .map(|(index, entry)| {
            let total = entry
                .total
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string());
            let slots = entry
                .slots
                .iter()
                .map(|slot| {
                    let value = slot
                        .value
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "-".to_string());
                    let lock = if slot.locked { " lock" } else { "" };
                    format!("#{} {}={}{}", slot.slot_id, slot.die_name, value, lock)
                })
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                "[{}] {} total:{} | {}",
                index + 1,
                entry.tray_name,
                total,
                slots
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
