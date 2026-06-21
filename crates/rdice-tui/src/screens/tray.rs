use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App, tray_name: &str) {
    let body = app
        .engine
        .get_tray(tray_name)
        .map(|tray| {
            if tray.slots.is_empty() {
                return "No dice in this tray. Press a to add one.".to_string();
            }

            tray.slots
                .iter()
                .map(|slot| {
                    let lock = if slot.locked { " LOCK" } else { "" };
                    let value = slot
                        .current_value
                        .as_ref()
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    format!(
                        "[{}] {}{} value {}",
                        slot.slot_id, slot.die_name, lock, value
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_else(|| "Tray not found".to_string());

    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(format!(" Tray: {tray_name} "))
                .borders(Borders::ALL),
        ),
        area,
    );
}

pub fn render_add_die(frame: &mut Frame<'_>, area: Rect, app: &App, tray_name: &str) {
    let body = app
        .engine
        .list_dice()
        .iter()
        .skip(app.add_die_page * 9)
        .take(9)
        .enumerate()
        .map(|(index, die)| format!("[{}] {}", index + 1, die.name))
        .collect::<Vec<_>>()
        .join("\n");
    let body = if body.is_empty() {
        "No dice on this page. Use PgUp/PgDn to navigate.".to_string()
    } else {
        body
    };

    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(format!(" Add die to {tray_name} "))
                .borders(Borders::ALL),
        ),
        area,
    );
}
