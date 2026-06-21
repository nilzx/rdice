use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let body = app
        .engine
        .list_trays()
        .iter()
        .skip(app.tray_manager_page * 9)
        .take(9)
        .enumerate()
        .map(|(index, tray)| format!("[{}] {}: {} slots", index + 1, tray.name, tray.slots.len()))
        .collect::<Vec<_>>()
        .join("\n");
    let body = if body.is_empty() {
        "No trays on this page. Press n to create one.".to_string()
    } else {
        body
    };
    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(" Tray Manager ")
                .borders(Borders::ALL),
        ),
        area,
    );
}
