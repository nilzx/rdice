use std::collections::BTreeMap;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_core::engine::DiceEngine;
use rdice_core::error::Result;

use crate::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverviewOptions {
    pub text_visible: bool,
    pub range_visible: bool,
    pub ev_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayCard {
    pub page_id: usize,
    pub name: String,
    pub selected: bool,
    pub composition: String,
    pub total_line: String,
    pub text_line: Option<String>,
}

pub fn build_tray_cards(
    engine: &DiceEngine,
    selected_trays: &[String],
    page: usize,
    options: OverviewOptions,
) -> Result<Vec<TrayCard>> {
    let start = page * 9;
    let trays = engine.list_trays().iter().skip(start).take(9);
    let mut cards = Vec::new();

    for (offset, tray) in trays.enumerate() {
        let die_names = tray
            .slots
            .iter()
            .map(|slot| slot.die_name.clone())
            .collect::<Vec<_>>();
        let analysis = engine.analyze_roll(&die_names, &[])?;
        let numeric_total = tray
            .slots
            .iter()
            .filter_map(|slot| match &slot.current_value {
                Some(FaceValue::Integer(value)) => Some(*value),
                _ => None,
            })
            .reduce(|acc, value| acc + value);
        let text_values = tray
            .slots
            .iter()
            .filter_map(|slot| match &slot.current_value {
                Some(FaceValue::Text(value)) => Some(value.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let range = options
            .range_visible
            .then_some((analysis.point_range.min, analysis.point_range.max));
        let ev = options.ev_visible.then_some(analysis.expected_value);

        cards.push(TrayCard {
            page_id: offset + 1,
            name: tray.name.clone(),
            selected: selected_trays.iter().any(|selected| selected == &tray.name),
            composition: format_composition(&die_names),
            total_line: format_total_line(
                numeric_total,
                range,
                ev,
                !options.text_visible && !text_values.is_empty(),
            ),
            text_line: (options.text_visible && !text_values.is_empty())
                .then(|| format!("Text:{}", text_values.join(","))),
        });
    }

    Ok(cards)
}

pub fn format_total_line(
    total: Option<i64>,
    range: Option<(i64, i64)>,
    ev: Option<f64>,
    hidden_text: bool,
) -> String {
    let analysis = match (range, ev) {
        (Some((min, max)), Some(ev)) => format!("({min}-{max}~{})", format_ev(ev)),
        (Some((min, max)), None) => format!("({min}-{max})"),
        (None, Some(ev)) => format!("(~{})", format_ev(ev)),
        (None, None) => String::new(),
    };
    let total = total
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let suffix = if hidden_text { " +t" } else { "" };
    format!("Total{analysis}:{total}{suffix}")
}

pub fn format_ev(value: f64) -> String {
    format!("{value:.1}")
}

fn format_composition(die_names: &[String]) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for die_name in die_names {
        *counts.entry(display_die_name(die_name)).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(name, count)| {
            if count == 1 {
                name
            } else {
                format!("{count}x{name}")
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_die_name(name: &str) -> String {
    let stripped = name.strip_prefix(CUSTOM_PREFIX).unwrap_or(name);
    let stripped = if stripped.len() > 1
        && stripped.starts_with('D')
        && stripped[1..].chars().all(|ch| ch.is_ascii_digit())
    {
        stripped.to_ascii_lowercase()
    } else {
        stripped.to_string()
    };
    let mut chars = stripped.chars();
    let prefix = chars.by_ref().take(7).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}...")
    } else {
        prefix
    }
}

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let options = OverviewOptions {
        text_visible: app.overview_text_visible,
        range_visible: app.overview_range_visible,
        ev_visible: app.overview_ev_visible,
    };
    let cards = build_tray_cards(&app.engine, &app.selected_trays, app.overview_page, options)
        .unwrap_or_default();

    frame.render_widget(Paragraph::new(render_grid_text(&cards, area.width)), area);
}

pub fn render_grid_text(cards: &[TrayCard], width: u16) -> String {
    if cards.is_empty() {
        return String::new();
    }

    let columns = 3_usize;
    let cell_width = ((width as usize).saturating_sub(columns + 1) / columns).max(18);
    let mut output = Vec::new();

    for row_cards in cards.chunks(columns) {
        let border = grid_border(row_cards.len(), cell_width);
        output.push(border.clone());
        for line_index in 0..4 {
            let cells = row_cards
                .iter()
                .map(|card| {
                    let text = card_line(card, line_index, cell_width);
                    pad_cell(&text, cell_width)
                })
                .collect::<Vec<_>>()
                .join("|");
            output.push(format!("|{cells}|"));
        }
        output.push(border.clone());
    }

    output.join("\n")
}

fn grid_border(columns: usize, cell_width: usize) -> String {
    format!(
        "+{}+",
        std::iter::repeat_n("-".repeat(cell_width), columns)
            .collect::<Vec<_>>()
            .join("+")
    )
}

fn card_line(card: &TrayCard, line_index: usize, width: usize) -> String {
    match line_index {
        0 => {
            let marker = if card.selected { " *" } else { "" };
            format!("[{}] {}{}", card.page_id, card.name, marker)
        }
        1 => card.composition.clone(),
        2 => right_align_text_marker(&card.total_line, width),
        3 => card.text_line.clone().unwrap_or_default(),
        _ => String::new(),
    }
}

fn right_align_text_marker(line: &str, width: usize) -> String {
    let Some(base) = line.strip_suffix(" +t") else {
        return line.to_string();
    };
    if base.len() + 3 >= width {
        return line.to_string();
    }
    format!("{base}{}+t", " ".repeat(width - base.len() - 2))
}

fn pad_cell(text: &str, width: usize) -> String {
    let truncated = truncate(text, width);
    format!("{truncated:<width$}")
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
