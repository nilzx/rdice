use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::Paragraph;

use crate::app::{App, DiceCreationStep, DiceCreationWizard, Screen};
use crate::screens::{dice_manager, history, overview, tray, tray_manager};
use crate::theme;

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

    let footer_style = if app.dice_creation.is_some() || app.command_buffer.is_some() {
        theme::command_footer(app.color_enabled)
    } else if app.message.is_some() {
        theme::message_footer(app.color_enabled)
    } else {
        theme::footer(app.color_enabled)
    };

    frame.render_widget(
        Paragraph::new(footer_text(app, area.width as usize)).style(footer_style),
        chunks[1],
    );
}

pub fn help_text(app: &App) -> &'static str {
    match app.screen {
        Screen::Overview => {
            "1-9 select  o<num> open  r roll  t/R/E display  h history  m trays  c dice  :  q"
        }
        Screen::TrayDetail(_) => {
            "r roll  a add  l<num> lock  d<num> remove  h history  m/c dice  Esc overview  :"
        }
        Screen::AddDie(_) => "1-9 add die  PgUp/PgDn pages  c dice  Esc tray",
        Screen::TrayManager => "n new  d<num> delete  e<num> edit  c dice  :tray ...  Esc overview",
        Screen::DiceManager => "n new  d<num> delete  e<num> edit  :dice ...  Esc back",
        Screen::History => "recent rolls  Esc back",
    }
}

pub fn footer_text(app: &App, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    if let Some(wizard) = &app.dice_creation {
        let (left, right) = dice_creation_footer_parts(wizard);
        return align_footer_parts(&left, &right, width);
    }

    let feedback = app
        .command_buffer
        .as_ref()
        .map(|buffer| command_feedback_text(buffer))
        .or_else(|| app.message.clone())
        .unwrap_or_default();
    let help = help_text(app);

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

fn align_footer_parts(left: &str, right: &str, width: usize) -> String {
    if right.is_empty() {
        return truncate(left, width);
    }
    if left.len() + right.len() <= width {
        return format!(
            "{left}{}{}",
            " ".repeat(width - left.len() - right.len()),
            right
        );
    }
    if right.len() + 1 < width {
        let left_width = width - right.len() - 1;
        return format!("{} {right}", truncate(left, left_width));
    }

    truncate(left, width)
}

fn command_feedback_text(buffer: &str) -> String {
    if let Some(input) = buffer.strip_prefix("dice edit ") {
        return guided_command_text("EDIT DIE", input, "<name> <faces...>", "save");
    }
    if let Some(input) = buffer.strip_prefix("tray new ") {
        return guided_command_text("NEW TRAY", input, "<name>", "create");
    }
    if let Some(input) = buffer.strip_prefix("tray rename ") {
        return guided_command_text("RENAME TRAY", input, "<old> <new>", "save");
    }
    if let Some(input) = buffer.strip_prefix("dice delete ") {
        return guided_command_text("DELETE DIE", input, "<name>", "delete");
    }
    if let Some(input) = buffer.strip_prefix("tray delete ") {
        return guided_command_text("DELETE TRAY", input, "<name>", "delete");
    }

    format!("COMMAND: {buffer}|  Enter apply  Esc cancel")
}

fn dice_creation_footer_parts(wizard: &DiceCreationWizard) -> (String, String) {
    let (left, right) = match wizard.step {
        DiceCreationStep::Name => (
            format!("NEW DIE > name: {}", field_text(&wizard.name, "name")),
            "Enter next  Esc cancel".to_string(),
        ),
        DiceCreationStep::FaceCount => (
            format!(
                "NEW DIE > face count: {}",
                field_text(&wizard.face_count, "count")
            ),
            "Enter next  Esc cancel".to_string(),
        ),
        DiceCreationStep::Face => {
            let target_count = wizard.target_face_count().unwrap_or(0);
            let action = if wizard.current_face_number() == target_count {
                "create"
            } else {
                "next"
            };
            (
                format!(
                    "NEW DIE > face {}/{}: {}",
                    wizard.current_face_number(),
                    target_count,
                    field_text(&wizard.current_face, "face")
                ),
                format!("Enter {action}  Esc cancel"),
            )
        }
    };

    if let Some(error) = &wizard.error {
        (format!("{left}  Error: {error}"), right)
    } else {
        (left, right)
    }
}

fn field_text(input: &str, placeholder: &str) -> String {
    if input.is_empty() {
        format!("<{placeholder}>|")
    } else {
        format!("{input}|")
    }
}

fn guided_command_text(label: &str, input: &str, placeholder: &str, action: &str) -> String {
    let shown_input = if input.is_empty() {
        format!("{placeholder} |")
    } else {
        format!("{input}|")
    };
    format!("{label}: {shown_input}  Enter {action}  Esc cancel")
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
