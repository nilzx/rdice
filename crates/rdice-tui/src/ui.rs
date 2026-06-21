use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::Paragraph;

use crate::app::{App, Screen};
use crate::screens::{dice_manager, history, overview, tray, tray_manager};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    match &app.screen {
        Screen::Overview => overview::render(frame, chunks[0], app),
        Screen::TrayDetail(name) => tray::render(frame, chunks[0], app, name),
        Screen::AddDie(name) => tray::render_add_die(frame, chunks[0], app, name),
        Screen::DiceManager => dice_manager::render(frame, chunks[0], app),
        Screen::TrayManager => tray_manager::render(frame, chunks[0], app),
        Screen::History => history::render(frame, chunks[0], app),
    }

    frame.render_widget(
        Paragraph::new(footer_text(app, area.width as usize)),
        chunks[1],
    );
}

pub fn help_text(app: &App) -> &'static str {
    match app.screen {
        Screen::Overview => {
            "1-9 select  o<num> open  r roll  t/R/E display  h history  m trays  :  q"
        }
        Screen::TrayDetail(_) => {
            "r roll  a add  l<num> lock  d<num> remove  h history  m dice  Esc overview  :"
        }
        Screen::AddDie(_) => "1-9 add die  PgUp/PgDn pages  Esc tray",
        Screen::TrayManager => "n new  d<num> delete  e<num> edit  :tray ...  Esc overview",
        Screen::DiceManager => "n new  d<num> delete  e<num> edit  :dice ...  Esc back",
        Screen::History => "recent rolls  Esc back",
    }
}

pub fn footer_text(app: &App, width: usize) -> String {
    let feedback = app
        .command_buffer
        .as_ref()
        .map(|buffer| format!("COMMAND: {buffer}  Enter apply  Esc cancel"))
        .or_else(|| app.message.clone())
        .unwrap_or_default();
    let help = help_text(app);

    if width == 0 {
        return String::new();
    }

    if feedback.is_empty() {
        return truncate(help, width);
    }

    let needed = feedback.len() + 2 + help.len();
    if needed <= width {
        return format!(
            "{feedback}{}{}",
            " ".repeat(width - feedback.len() - help.len()),
            help
        );
    }

    truncate(&feedback, width)
}

fn truncate(text: &str, width: usize) -> String {
    let mut output = String::new();
    for ch in text.chars() {
        if output.len() + ch.len_utf8() > width {
            break;
        }
        output.push(ch);
    }
    output
}
