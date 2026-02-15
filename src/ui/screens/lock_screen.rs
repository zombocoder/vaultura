use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::theme;
use crate::ui::{Action, Component};

pub struct LockScreen {
    password_input: String,
    error_message: Option<String>,
    vault_exists: bool,
}

impl LockScreen {
    pub fn new(vault_exists: bool) -> Self {
        Self {
            password_input: String::new(),
            error_message: None,
            vault_exists,
        }
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }

    pub fn clear(&mut self) {
        self.password_input.clear();
        self.error_message = None;
    }

    pub fn set_vault_exists(&mut self, exists: bool) {
        self.vault_exists = exists;
    }
}

impl Component for LockScreen {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
            (KeyCode::Enter, _) => {
                if self.password_input.is_empty() {
                    self.error_message = Some("Password cannot be empty".to_string());
                    Action::None
                } else {
                    let pw = self.password_input.clone();
                    self.error_message = None;
                    if self.vault_exists {
                        Action::UnlockVault(pw)
                    } else {
                        Action::CreateVault(pw)
                    }
                }
            }
            (KeyCode::Char(c), _) => {
                self.password_input.push(c);
                self.error_message = None;
                Action::None
            }
            (KeyCode::Backspace, _) => {
                self.password_input.pop();
                self.error_message = None;
                Action::None
            }
            (KeyCode::Esc, _) => Action::Quit,
            _ => Action::None,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);

        // Center a box in the middle of the screen
        let box_width = 50u16.min(area.width.saturating_sub(4));
        let box_height = 10u16.min(area.height.saturating_sub(2));

        let vertical = Layout::vertical([Constraint::Length(box_height)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Length(box_width)]).flex(Flex::Center);
        let [vert_area] = vertical.areas(area);
        let [center_area] = horizontal.areas(vert_area);

        let title = if self.vault_exists {
            " Unlock Vault "
        } else {
            " Create New Vault "
        };

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme::style_border(true));

        let inner = block.inner(center_area);
        frame.render_widget(block, center_area);

        let chunks = Layout::vertical([
            Constraint::Length(2), // Logo/title
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Label
            Constraint::Length(3), // Password input
            Constraint::Length(1), // Error message
            Constraint::Min(0),    // Hint
        ])
        .split(inner);

        // Logo
        let logo = Paragraph::new("🔒 VAULTURA")
            .alignment(Alignment::Center)
            .style(theme::style_accent());
        frame.render_widget(logo, chunks[0]);

        // Label
        let label = if self.vault_exists {
            "Enter master password:"
        } else {
            "Choose a master password:"
        };
        let label_para = Paragraph::new(label).style(theme::style_default());
        frame.render_widget(label_para, chunks[2]);

        // Password input (masked)
        let masked: String = "•".repeat(self.password_input.len());
        let display = if self.password_input.is_empty() {
            Span::styled("type your password...", theme::style_muted())
        } else {
            Span::styled(masked, theme::style_default())
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::style_border(true));
        let input = Paragraph::new(Line::from(display)).block(input_block);
        frame.render_widget(input, chunks[3]);

        // Error message
        if let Some(ref err) = self.error_message {
            let err_para = Paragraph::new(err.as_str()).style(theme::style_error());
            frame.render_widget(err_para, chunks[4]);
        }

        // Hint
        let hint = Paragraph::new("Enter ↵ submit  |  Esc/Ctrl+C quit")
            .alignment(Alignment::Center)
            .style(theme::style_muted());
        frame.render_widget(hint, chunks[5]);
    }
}
