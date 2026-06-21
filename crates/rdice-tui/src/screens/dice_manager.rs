use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let body = app
        .engine
        .custom_dice()
        .into_iter()
        .take(9)
        .enumerate()
        .map(|(index, die)| {
            let name = die.name.strip_prefix("✽").unwrap_or(&die.name);
            format!("[{}] {}: {} faces", index + 1, name, die.faces.len())
        })
        .collect::<Vec<_>>()
        .join("\n");
    frame.render_widget(
        Paragraph::new(body).block(
            Block::default()
                .title(" Dice Manager ")
                .borders(Borders::ALL),
        ),
        area,
    );
}
