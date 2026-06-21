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
                    .title(" Dice Manager ")
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
        .custom_dice()
        .into_iter()
        .skip(app.dice_manager_page * 9)
        .take(9)
        .enumerate()
        .map(|(index, die)| {
            let name = die.name.strip_prefix("✽").unwrap_or(&die.name);
            let global_id = app.dice_manager_page * 9 + index + 1;
            format!(
                "[{}|{}] {}: {} faces",
                global_id,
                index + 1,
                name,
                die.faces.len()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if body.is_empty() {
        "No custom dice. Press n to create one.".to_string()
    } else {
        body
    }
}
