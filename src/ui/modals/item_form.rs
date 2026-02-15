use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use uuid::Uuid;

use crate::core::models::{Group, Item};
use crate::core::vault_service::ItemDraft;
use crate::ui::theme;
use crate::ui::{Action, Component};

const FIELD_COUNT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Title,
    Username,
    Password,
    Url,
    Notes,
    Tags,
    Group,
}

const FIELDS: [Field; FIELD_COUNT] = [
    Field::Title,
    Field::Username,
    Field::Password,
    Field::Url,
    Field::Notes,
    Field::Tags,
    Field::Group,
];

pub struct ItemForm {
    editing_id: Option<Uuid>,
    field_values: [String; FIELD_COUNT],
    current_field: usize,
    groups: Vec<(Uuid, String)>,
    selected_group_index: Option<usize>, // None = no group
}

impl ItemForm {
    pub fn new_create(groups: &[Group], default_group: Option<Uuid>) -> Self {
        let group_list: Vec<(Uuid, String)> =
            groups.iter().map(|g| (g.id, g.name.clone())).collect();
        let selected_group_index =
            default_group.and_then(|gid| group_list.iter().position(|g| g.0 == gid));

        Self {
            editing_id: None,
            field_values: Default::default(),
            current_field: 0,
            groups: group_list,
            selected_group_index,
        }
    }

    pub fn new_edit(item: &Item, groups: &[Group]) -> Self {
        let group_list: Vec<(Uuid, String)> =
            groups.iter().map(|g| (g.id, g.name.clone())).collect();
        let selected_group_index = item
            .group_id
            .and_then(|gid| group_list.iter().position(|g| g.0 == gid));

        let field_values = [
            item.title.clone(),
            item.username.clone(),
            item.password.clone(),
            item.url.clone(),
            item.notes.clone(),
            item.tags.join(", "),
            String::new(), // Group handled by selected_group_index
        ];

        Self {
            editing_id: Some(item.id),
            field_values,
            current_field: 0,
            groups: group_list,
            selected_group_index,
        }
    }

    pub fn set_password(&mut self, password: String) {
        self.field_values[2] = password;
    }

    fn current_value(&mut self) -> &mut String {
        &mut self.field_values[self.current_field]
    }

    fn build_draft(&self) -> ItemDraft {
        let tags: Vec<String> = self.field_values[5]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let group_id = self
            .selected_group_index
            .and_then(|i| self.groups.get(i).map(|(id, _)| *id));

        ItemDraft {
            title: self.field_values[0].clone(),
            username: self.field_values[1].clone(),
            password: self.field_values[2].clone(),
            url: self.field_values[3].clone(),
            notes: self.field_values[4].clone(),
            tags,
            group_id,
        }
    }

    fn field_label(field: Field) -> &'static str {
        match field {
            Field::Title => "Title",
            Field::Username => "Username",
            Field::Password => "Password",
            Field::Url => "URL",
            Field::Notes => "Notes",
            Field::Tags => "Tags (comma-separated)",
            Field::Group => "Group",
        }
    }
}

impl Component for ItemForm {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Action::CloseModal,
            (KeyCode::Tab, _) | (KeyCode::Down, _) => {
                self.current_field = (self.current_field + 1) % FIELD_COUNT;
                Action::None
            }
            (KeyCode::BackTab, _) | (KeyCode::Up, _) => {
                self.current_field = if self.current_field == 0 {
                    FIELD_COUNT - 1
                } else {
                    self.current_field - 1
                };
                Action::None
            }
            (KeyCode::Enter, KeyModifiers::CONTROL)
            | (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if self.field_values[0].is_empty() {
                    Action::SetStatus("Title is required".to_string())
                } else {
                    let draft = self.build_draft();
                    match self.editing_id {
                        Some(id) => Action::UpdateItem(id, draft),
                        None => Action::CreateItem(draft),
                    }
                }
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => Action::OpenPasswordGenerator,
            _ => {
                // Group field uses left/right to cycle
                if FIELDS[self.current_field] == Field::Group {
                    match key.code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            self.selected_group_index = match self.selected_group_index {
                                None => {
                                    if self.groups.is_empty() {
                                        None
                                    } else {
                                        Some(self.groups.len() - 1)
                                    }
                                }
                                Some(0) => None,
                                Some(i) => Some(i - 1),
                            };
                            Action::None
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            self.selected_group_index = match self.selected_group_index {
                                None => {
                                    if self.groups.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    }
                                }
                                Some(i) if i + 1 >= self.groups.len() => None,
                                Some(i) => Some(i + 1),
                            };
                            Action::None
                        }
                        _ => Action::None,
                    }
                } else {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.current_value().push(c);
                            Action::None
                        }
                        KeyCode::Backspace => {
                            self.current_value().pop();
                            Action::None
                        }
                        _ => Action::None,
                    }
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 60u16.min(area.width.saturating_sub(4));
        let height = (FIELD_COUNT as u16 * 3 + 6).min(area.height.saturating_sub(2));

        let vert = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
        let horiz = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
        let [v] = vert.areas(area);
        let [center] = horiz.areas(v);

        frame.render_widget(Clear, center);

        let title = if self.editing_id.is_some() {
            " Edit Item "
        } else {
            " New Item "
        };

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme::style_border(true));

        let inner = block.inner(center);
        frame.render_widget(block, center);

        let mut constraints: Vec<Constraint> =
            FIELDS.iter().map(|_| Constraint::Length(3)).collect();
        constraints.push(Constraint::Length(2)); // hints
        constraints.push(Constraint::Min(0));

        let chunks = Layout::vertical(constraints).split(inner);

        for (i, field) in FIELDS.iter().enumerate() {
            let is_current = i == self.current_field;
            let label = Self::field_label(*field);

            let value_display = if *field == Field::Group {
                match self.selected_group_index {
                    None => "< None >".to_string(),
                    Some(idx) => format!("< {} >", self.groups[idx].1),
                }
            } else {
                let val = &self.field_values[i];
                if val.is_empty() {
                    format!("{label}...")
                } else if *field == Field::Password && !is_current {
                    theme::PASSWORD_MASK.to_string()
                } else {
                    val.clone()
                }
            };

            let style = if is_current {
                theme::style_accent()
            } else {
                theme::style_muted()
            };

            let field_block = Block::default()
                .title(format!(" {label} "))
                .title_style(if is_current {
                    theme::style_accent()
                } else {
                    theme::style_muted()
                })
                .borders(Borders::ALL)
                .border_style(theme::style_border(is_current));

            let content = if is_current && *field != Field::Group {
                Line::from(vec![
                    Span::raw(&value_display),
                    Span::styled("█", theme::style_accent()),
                ])
            } else {
                let text_style = if self.field_values[i].is_empty() && *field != Field::Group {
                    theme::style_muted()
                } else {
                    style
                };
                Line::from(Span::styled(value_display, text_style))
            };

            let para = Paragraph::new(content).block(field_block);
            frame.render_widget(para, chunks[i]);
        }

        // Hints
        let hints = Paragraph::new(Line::from(vec![
            Span::styled("Tab", theme::style_accent()),
            Span::raw(" next  "),
            Span::styled("Ctrl+S", theme::style_accent()),
            Span::raw(" save  "),
            Span::styled("Ctrl+P", theme::style_accent()),
            Span::raw(" gen pw  "),
            Span::styled("Esc", theme::style_accent()),
            Span::raw(" cancel"),
        ]))
        .style(theme::style_muted());
        frame.render_widget(hints, chunks[FIELD_COUNT]);
    }
}
