use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let body = app
        .engine
        .list_trays()
        .iter()
        .take(9)
        .enumerate()
        .map(|(index, tray)| format!("[{}] {}: {} slots", index + 1, tray.name, tray.slots.len()))
        .collect::<Vec<_>>()
        .join("\n");
    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(" Tray Manager ")
                .borders(Borders::ALL),
        ),
        area,
    );
}
