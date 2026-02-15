use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::core::password_generator::{self, PasswordConfig};
use crate::ui::theme;
use crate::ui::{Action, Component};

const OPTION_COUNT: usize = 6;

pub struct PasswordGeneratorModal {
    config: PasswordConfig,
    generated: String,
    current_option: usize,
}

impl Default for PasswordGeneratorModal {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordGeneratorModal {
    pub fn new() -> Self {
        let config = PasswordConfig::default();
        let generated = password_generator::generate_password(&config);
        Self {
            config,
            generated,
            current_option: 0,
        }
    }

    fn regenerate(&mut self) {
        self.generated = password_generator::generate_password(&self.config);
    }

    pub fn generated_password(&self) -> &str {
        &self.generated
    }
}

impl Component for PasswordGeneratorModal {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Action::CloseModal,
            (KeyCode::Char('r'), _) => {
                self.regenerate();
                Action::None
            }
            (KeyCode::Enter, KeyModifiers::CONTROL) | (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                // "Use" the generated password
                Action::UseGeneratedPassword
            }
            (KeyCode::Tab, _) | (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                self.current_option = (self.current_option + 1) % OPTION_COUNT;
                Action::None
            }
            (KeyCode::BackTab, _) | (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                self.current_option = if self.current_option == 0 {
                    OPTION_COUNT - 1
                } else {
                    self.current_option - 1
                };
                Action::None
            }
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                match self.current_option {
                    0 => {
                        // Length: increase by 1
                        if self.config.length < 128 {
                            self.config.length += 1;
                        }
                    }
                    1 => self.config.uppercase = !self.config.uppercase,
                    2 => self.config.lowercase = !self.config.lowercase,
                    3 => self.config.digits = !self.config.digits,
                    4 => self.config.symbols = !self.config.symbols,
                    5 => self.config.exclude_ambiguous = !self.config.exclude_ambiguous,
                    _ => {}
                }
                self.regenerate();
                Action::None
            }
            (KeyCode::Left | KeyCode::Char('h'), _) => {
                if self.current_option == 0 && self.config.length > 4 {
                    self.config.length -= 1;
                    self.regenerate();
                }
                Action::None
            }
            (KeyCode::Right | KeyCode::Char('l'), _) => {
                if self.current_option == 0 && self.config.length < 128 {
                    self.config.length += 1;
                    self.regenerate();
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 50u16.min(area.width.saturating_sub(4));
        let height = 18u16.min(area.height.saturating_sub(2));

        let vert = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
        let horiz = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
        let [v] = vert.areas(area);
        let [center] = horiz.areas(v);

        frame.render_widget(Clear, center);

        let block = Block::default()
            .title(" Password Generator ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme::style_border(true));

        let inner = block.inner(center);
        frame.render_widget(block, center);

        let chunks = Layout::vertical([
            Constraint::Length(3), // Generated password
            Constraint::Length(1), // Spacer
            Constraint::Min(1),   // Options
            Constraint::Length(2), // Hints
        ])
        .split(inner);

        // Generated password
        let pw_block = Block::default()
            .title(" Generated ")
            .borders(Borders::ALL)
            .border_style(theme::style_border(false));
        let pw = Paragraph::new(self.generated.as_str())
            .style(theme::style_accent())
            .block(pw_block);
        frame.render_widget(pw, chunks[0]);

        // Options
        let options = [
            (
                format!("Length: {}", self.config.length),
                true,
                "← →",
            ),
            (
                "Uppercase (A-Z)".to_string(),
                self.config.uppercase,
                "",
            ),
            (
                "Lowercase (a-z)".to_string(),
                self.config.lowercase,
                "",
            ),
            (
                "Digits (0-9)".to_string(),
                self.config.digits,
                "",
            ),
            (
                "Symbols (!@#...)".to_string(),
                self.config.symbols,
                "",
            ),
            (
                "Exclude ambiguous (0OlI1)".to_string(),
                self.config.exclude_ambiguous,
                "",
            ),
        ];

        let option_lines: Vec<Line> = options
            .iter()
            .enumerate()
            .map(|(i, (label, enabled, hint))| {
                let marker = if i == 0 {
                    format!("  {} ", hint)
                } else if *enabled {
                    "  [x] ".to_string()
                } else {
                    "  [ ] ".to_string()
                };

                let style = if i == self.current_option {
                    theme::style_selected()
                } else {
                    theme::style_default()
                };

                Line::from(Span::styled(format!("{marker}{label}"), style))
            })
            .collect();

        let options_para = Paragraph::new(option_lines);
        frame.render_widget(options_para, chunks[2]);

        // Hints
        let hints = Paragraph::new(Line::from(vec![
            Span::styled("[r]", theme::style_accent()),
            Span::raw(" regenerate  "),
            Span::styled("[Space]", theme::style_accent()),
            Span::raw(" toggle  "),
            Span::styled("Ctrl+S", theme::style_accent()),
            Span::raw(" use  "),
            Span::styled("Esc", theme::style_accent()),
            Span::raw(" cancel"),
        ]))
        .style(theme::style_muted());
        frame.render_widget(hints, chunks[3]);
    }
}
