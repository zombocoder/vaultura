use ratatui::style::{Color, Modifier, Style};

// Color palette
pub const BG: Color = Color::Reset;
pub const FG: Color = Color::White;
pub const ACCENT: Color = Color::Cyan;
pub const ACCENT_DIM: Color = Color::DarkGray;
pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 60);
pub const ERROR_FG: Color = Color::Red;
pub const SUCCESS_FG: Color = Color::Green;
pub const WARNING_FG: Color = Color::Yellow;
pub const BORDER: Color = Color::DarkGray;
pub const BORDER_FOCUSED: Color = Color::Cyan;
pub const MUTED: Color = Color::DarkGray;
pub const PASSWORD_MASK: &str = "••••••••••••";

// Reusable styles
pub fn style_default() -> Style {
    Style::default().fg(FG).bg(BG)
}

pub fn style_accent() -> Style {
    Style::default().fg(ACCENT)
}

pub fn style_muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn style_error() -> Style {
    Style::default().fg(ERROR_FG)
}

pub fn style_success() -> Style {
    Style::default().fg(SUCCESS_FG)
}

pub fn style_warning() -> Style {
    Style::default().fg(WARNING_FG)
}

pub fn style_selected() -> Style {
    Style::default().fg(FG).bg(HIGHLIGHT_BG).add_modifier(Modifier::BOLD)
}

pub fn style_border(focused: bool) -> Style {
    if focused {
        Style::default().fg(BORDER_FOCUSED)
    } else {
        Style::default().fg(BORDER)
    }
}

pub fn style_title(focused: bool) -> Style {
    if focused {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(FG).add_modifier(Modifier::BOLD)
    }
}
