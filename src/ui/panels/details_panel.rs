use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use uuid::Uuid;

use crate::core::models::Item;
use crate::ui::theme;
use crate::ui::{Action, Component};

pub struct DetailsPanel {
    item: Option<DisplayItem>,
    show_password: bool,
    focused: bool,
    scroll_offset: u16,
}

#[derive(Clone)]
struct DisplayItem {
    id: Uuid,
    title: String,
    username: String,
    password: String,
    url: String,
    notes: String,
    tags: Vec<String>,
    group_name: String,
    created_at: String,
    modified_at: String,
    password_history_count: usize,
}

impl Default for DetailsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl DetailsPanel {
    pub fn new() -> Self {
        Self {
            item: None,
            show_password: false,
            focused: false,
            scroll_offset: 0,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_item(&mut self, item: Option<&Item>, group_name: &str) {
        self.show_password = false;
        self.scroll_offset = 0;
        self.item = item.map(|i| DisplayItem {
            id: i.id,
            title: i.title.clone(),
            username: i.username.clone(),
            password: i.password.clone(),
            url: i.url.clone(),
            notes: i.notes.clone(),
            tags: i.tags.clone(),
            group_name: group_name.to_string(),
            created_at: i.created_at.format("%Y-%m-%d %H:%M").to_string(),
            modified_at: i.modified_at.format("%Y-%m-%d %H:%M").to_string(),
            password_history_count: i.password_history.len(),
        });
    }

    pub fn clear(&mut self) {
        self.item = None;
        self.show_password = false;
        self.scroll_offset = 0;
    }

    pub fn selected_item_id(&self) -> Option<Uuid> {
        self.item.as_ref().map(|i| i.id)
    }
}

impl Component for DetailsPanel {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if !self.focused {
            return Action::None;
        }

        match key.code {
            KeyCode::Char('r') => {
                self.show_password = !self.show_password;
                Action::None
            }
            KeyCode::Char('p') => {
                if let Some(ref item) = self.item {
                    Action::CopyPassword(item.id)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('u') => {
                if let Some(ref item) = self.item {
                    Action::CopyUsername(item.id)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('e') => {
                if let Some(ref item) = self.item {
                    Action::OpenEditItemForm(item.id)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('d') => {
                if let Some(ref item) = self.item {
                    Action::OpenDeleteConfirm(item.id)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                Action::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Action::None
            }
            _ => Action::None,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Details ")
            .title_style(theme::style_title(self.focused))
            .borders(Borders::ALL)
            .border_style(theme::style_border(self.focused));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(ref item) = self.item else {
            let empty =
                Paragraph::new("Select an item to view details").style(theme::style_muted());
            frame.render_widget(empty, inner);
            return;
        };

        let chunks = Layout::vertical([
            Constraint::Length(2), // Title
            Constraint::Min(1),    // Fields
            Constraint::Length(2), // Hints
        ])
        .split(inner);

        // Title
        let title = Paragraph::new(Line::from(vec![Span::styled(
            &item.title,
            theme::style_accent(),
        )]));
        frame.render_widget(title, chunks[0]);

        // Fields
        let password_display = if self.show_password {
            item.password.as_str()
        } else {
            theme::PASSWORD_MASK
        };

        let tags_display = if item.tags.is_empty() {
            "—".to_string()
        } else {
            item.tags.join(", ")
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Username:  ", theme::style_muted()),
                Span::raw(if item.username.is_empty() {
                    "—"
                } else {
                    &item.username
                }),
            ]),
            Line::from(vec![
                Span::styled("Password:  ", theme::style_muted()),
                Span::raw(password_display),
                Span::styled(
                    if self.show_password {
                        "  [r] hide"
                    } else {
                        "  [r] reveal"
                    },
                    theme::style_muted(),
                ),
            ]),
            Line::from(vec![
                Span::styled("URL:       ", theme::style_muted()),
                Span::raw(if item.url.is_empty() {
                    "—"
                } else {
                    &item.url
                }),
            ]),
            Line::from(vec![
                Span::styled("Group:     ", theme::style_muted()),
                Span::raw(&item.group_name),
            ]),
            Line::from(vec![
                Span::styled("Tags:      ", theme::style_muted()),
                Span::raw(&tags_display),
            ]),
            Line::raw(""),
            Line::from(vec![Span::styled("Notes:", theme::style_muted())]),
        ];

        if item.notes.is_empty() {
            lines.push(Line::from(Span::raw("  —")));
        } else {
            for line in item.notes.lines() {
                lines.push(Line::from(Span::raw(format!("  {line}"))));
            }
        }

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("Created:   ", theme::style_muted()),
            Span::raw(&item.created_at),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Modified:  ", theme::style_muted()),
            Span::raw(&item.modified_at),
        ]));

        if item.password_history_count > 0 {
            lines.push(Line::from(vec![
                Span::styled("History:   ", theme::style_muted()),
                Span::raw(format!(
                    "{} previous passwords",
                    item.password_history_count
                )),
            ]));
        }

        let fields = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));
        frame.render_widget(fields, chunks[1]);

        // Key hints
        let hints = Paragraph::new(Line::from(vec![
            Span::styled("[p]", theme::style_accent()),
            Span::raw(" copy pw  "),
            Span::styled("[u]", theme::style_accent()),
            Span::raw(" copy user  "),
            Span::styled("[e]", theme::style_accent()),
            Span::raw(" edit  "),
            Span::styled("[d]", theme::style_accent()),
            Span::raw(" delete"),
        ]))
        .style(theme::style_muted());
        frame.render_widget(hints, chunks[2]);
    }
}
