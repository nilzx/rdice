use std::collections::BTreeMap;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use rdice_core::die::{CUSTOM_PREFIX, FaceValue};
use rdice_core::engine::DiceEngine;
use rdice_core::error::Result;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::App;
use crate::theme;

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

    frame.render_widget(
        Paragraph::new(render_grid_lines(&cards, area.width, app.color_enabled))
            .style(theme::overview(app.color_enabled)),
        area,
    );
}

fn render_grid_lines(cards: &[TrayCard], width: u16, color_enabled: bool) -> Vec<Line<'static>> {
    render_grid_text(cards, width)
        .lines()
        .enumerate()
        .map(|(index, line)| style_grid_line(line, index % 6, color_enabled))
        .collect()
}

fn style_grid_line(line: &str, row_line_index: usize, color_enabled: bool) -> Line<'static> {
    if line.starts_with('+') {
        return Line::from(Span::styled(line.to_string(), theme::border(color_enabled)));
    }

    let content_style = match row_line_index {
        1 => theme::name(color_enabled),
        2 => theme::muted(color_enabled),
        3 => theme::value(color_enabled),
        4 => theme::text_value(color_enabled),
        _ => theme::content(color_enabled),
    };

    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '|' => spans.push(Span::styled("|".to_string(), theme::border(color_enabled))),
            '[' => {
                let mut token = String::from("[");
                for next in chars.by_ref() {
                    token.push(next);
                    if next == ']' {
                        break;
                    }
                }
                spans.push(Span::styled(token, theme::shortcut(color_enabled)));
            }
            '*' => spans.push(Span::styled(
                "*".to_string(),
                theme::selected(color_enabled),
            )),
            _ => {
                let mut text = ch.to_string();
                while let Some(next) = chars.peek().copied() {
                    if matches!(next, '|' | '[' | '*') {
                        break;
                    }
                    text.push(next);
                    chars.next();
                }
                spans.push(Span::styled(text, content_style));
            }
        }
    }

    Line::from(spans)
}

pub fn render_grid_text(cards: &[TrayCard], width: u16) -> String {
    if cards.is_empty() {
        return "No trays on this page. Press m to manage trays.".to_string();
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
    let base_width = UnicodeWidthStr::width(base);
    let marker_width = UnicodeWidthStr::width("+t");
    if base_width + 1 + marker_width >= width {
        return truncate(base, width);
    }
    format!("{base}{}+t", " ".repeat(width - base_width - marker_width))
}

fn pad_cell(text: &str, width: usize) -> String {
    let truncated = truncate(text, width);
    let padding = width.saturating_sub(UnicodeWidthStr::width(truncated.as_str()));
    format!("{truncated}{}", " ".repeat(padding))
}

fn truncate(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut used_width = 0;
    for ch in text.chars() {
        let ch_width = ch.width().unwrap_or(0);
        if used_width + ch_width > width {
            break;
        }
        output.push(ch);
        used_width += ch_width;
    }
    output
}
