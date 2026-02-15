use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme;
use crate::ui::{Action, Component};

pub struct ConfirmDialog {
    message: String,
    confirm_action: Action,
    selected: bool, // false = No (default), true = Yes
}

impl ConfirmDialog {
    pub fn new(message: String, confirm_action: Action) -> Self {
        Self {
            message,
            confirm_action,
            selected: false,
        }
    }
}

impl Component for ConfirmDialog {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Tab
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                self.selected = !self.selected;
                Action::None
            }
            KeyCode::Enter => {
                if self.selected {
                    self.confirm_action.clone()
                } else {
                    Action::CloseModal
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => self.confirm_action.clone(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CloseModal,
            _ => Action::None,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 40u16.min(area.width.saturating_sub(4));
        let height = 8u16.min(area.height.saturating_sub(2));

        let vert = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
        let horiz = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
        let [v] = vert.areas(area);
        let [center] = horiz.areas(v);

        frame.render_widget(Clear, center);

        let block = Block::default()
            .title(" Confirm ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme::style_border(true));

        let inner = block.inner(center);
        frame.render_widget(block, center);

        let chunks = Layout::vertical([
            Constraint::Length(2), // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
            Constraint::Min(0),
        ])
        .split(inner);

        let msg = Paragraph::new(self.message.as_str())
            .alignment(Alignment::Center)
            .style(theme::style_warning());
        frame.render_widget(msg, chunks[0]);

        let no_style = if !self.selected {
            theme::style_selected()
        } else {
            theme::style_muted()
        };
        let yes_style = if self.selected {
            theme::style_selected()
        } else {
            theme::style_muted()
        };

        let buttons = Line::from(vec![
            Span::styled("  [ No ]  ", no_style),
            Span::raw("    "),
            Span::styled("  [ Yes ]  ", yes_style),
        ]);
        let buttons_para = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_para, chunks[2]);
    }
}
