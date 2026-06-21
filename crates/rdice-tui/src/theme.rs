use ratatui::style::{Color, Modifier, Style};

pub fn color_enabled_from_env() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}

pub fn title(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Yellow).add_modifier(Modifier::BOLD)
}

pub fn border(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Cyan)
}

pub fn content(color_enabled: bool) -> Style {
    colored(color_enabled, Color::White)
}

pub fn overview(color_enabled: bool) -> Style {
    content(color_enabled)
}

pub fn shortcut(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Yellow).add_modifier(Modifier::BOLD)
}

pub fn name(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Cyan).add_modifier(Modifier::BOLD)
}

pub fn value(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Green)
}

pub fn text_value(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Magenta)
}

pub fn muted(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Gray)
}

pub fn selected(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Red).add_modifier(Modifier::BOLD)
}

pub fn footer(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Gray)
}

pub fn message_footer(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Green)
}

pub fn command_footer(color_enabled: bool) -> Style {
    colored(color_enabled, Color::Magenta).add_modifier(Modifier::BOLD)
}

fn colored(color_enabled: bool, color: Color) -> Style {
    if color_enabled {
        Style::default().fg(color)
    } else {
        Style::default()
    }
}
