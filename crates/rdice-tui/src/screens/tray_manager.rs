use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    frame.render_widget(
        Paragraph::new(render_text(app))
            .style(theme::content(app.color_enabled))
            .block(
                Block::default()
                    .title(" Tray Manager ")
                    .title_style(theme::title(app.color_enabled))
                    .borders(Borders::ALL)
                    .border_style(theme::border(app.color_enabled)),
            ),
        area,
    );
}

pub fn render_text(app: &App) -> String {
    let body = app
        .engine
        .list_trays()
        .iter()
        .skip(app.tray_manager_page * 9)
        .take(9)
        .enumerate()
        .map(|(index, tray)| {
            let global_id = app.tray_manager_page * 9 + index + 1;
            format!(
                "[{}|{}] {}: {} slots",
                global_id,
                index + 1,
                tray.name,
                tray.slots.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if body.is_empty() {
        "No trays on this page. Press n to create one.".to_string()
    } else {
        body
    }
}
