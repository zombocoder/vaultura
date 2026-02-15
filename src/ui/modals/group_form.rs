use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use uuid::Uuid;

use crate::core::models::Group;
use crate::ui::theme;
use crate::ui::{Action, Component};

pub struct GroupForm {
    editing_id: Option<Uuid>,
    name: String,
    parent_groups: Vec<(Uuid, String)>,
    selected_parent_index: Option<usize>,
    current_field: usize, // 0 = name, 1 = parent
}

impl GroupForm {
    pub fn new_create(groups: &[Group]) -> Self {
        let parent_groups: Vec<(Uuid, String)> =
            groups.iter().map(|g| (g.id, g.name.clone())).collect();
        Self {
            editing_id: None,
            name: String::new(),
            parent_groups,
            selected_parent_index: None,
            current_field: 0,
        }
    }

    pub fn new_edit(group: &Group, all_groups: &[Group]) -> Self {
        let parent_groups: Vec<(Uuid, String)> = all_groups
            .iter()
            .filter(|g| g.id != group.id)
            .map(|g| (g.id, g.name.clone()))
            .collect();
        let selected_parent_index = group
            .parent_id
            .and_then(|pid| parent_groups.iter().position(|g| g.0 == pid));

        Self {
            editing_id: Some(group.id),
            name: group.name.clone(),
            parent_groups,
            selected_parent_index,
            current_field: 0,
        }
    }
}

impl Component for GroupForm {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Action::CloseModal,
            (KeyCode::Tab, _) | (KeyCode::Down, _) => {
                self.current_field = (self.current_field + 1) % 2;
                Action::None
            }
            (KeyCode::BackTab, _) | (KeyCode::Up, _) => {
                self.current_field = if self.current_field == 0 { 1 } else { 0 };
                Action::None
            }
            (KeyCode::Enter, KeyModifiers::CONTROL)
            | (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if self.name.is_empty() {
                    Action::SetStatus("Group name is required".to_string())
                } else {
                    let parent_id = self
                        .selected_parent_index
                        .and_then(|i| self.parent_groups.get(i).map(|(id, _)| *id));
                    match self.editing_id {
                        Some(id) => Action::UpdateGroup(id, self.name.clone(), parent_id),
                        None => Action::CreateGroup(self.name.clone(), parent_id),
                    }
                }
            }
            _ => {
                if self.current_field == 0 {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.name.push(c);
                            Action::None
                        }
                        KeyCode::Backspace => {
                            self.name.pop();
                            Action::None
                        }
                        _ => Action::None,
                    }
                } else {
                    match key.code {
                        KeyCode::Left | KeyCode::Char('h') => {
                            self.selected_parent_index = match self.selected_parent_index {
                                None => {
                                    if self.parent_groups.is_empty() {
                                        None
                                    } else {
                                        Some(self.parent_groups.len() - 1)
                                    }
                                }
                                Some(0) => None,
                                Some(i) => Some(i - 1),
                            };
                            Action::None
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            self.selected_parent_index = match self.selected_parent_index {
                                None => {
                                    if self.parent_groups.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    }
                                }
                                Some(i) if i + 1 >= self.parent_groups.len() => None,
                                Some(i) => Some(i + 1),
                            };
                            Action::None
                        }
                        _ => Action::None,
                    }
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 50u16.min(area.width.saturating_sub(4));
        let height = 14u16.min(area.height.saturating_sub(2));

        let vert = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);
        let horiz = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);
        let [v] = vert.areas(area);
        let [center] = horiz.areas(v);

        frame.render_widget(Clear, center);

        let title = if self.editing_id.is_some() {
            " Edit Group "
        } else {
            " New Group "
        };

        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme::style_border(true));

        let inner = block.inner(center);
        frame.render_widget(block, center);

        let chunks = Layout::vertical([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Parent
            Constraint::Length(2), // Hints
            Constraint::Min(0),
        ])
        .split(inner);

        // Name field
        let name_focused = self.current_field == 0;
        let name_block = Block::default()
            .title(" Name ")
            .title_style(if name_focused {
                theme::style_accent()
            } else {
                theme::style_muted()
            })
            .borders(Borders::ALL)
            .border_style(theme::style_border(name_focused));

        let name_content = if name_focused {
            Line::from(vec![
                Span::raw(&self.name),
                Span::styled("█", theme::style_accent()),
            ])
        } else if self.name.is_empty() {
            Line::from(Span::styled("Group name...", theme::style_muted()))
        } else {
            Line::from(Span::raw(&self.name))
        };

        frame.render_widget(Paragraph::new(name_content).block(name_block), chunks[0]);

        // Parent field
        let parent_focused = self.current_field == 1;
        let parent_block = Block::default()
            .title(" Parent Group ")
            .title_style(if parent_focused {
                theme::style_accent()
            } else {
                theme::style_muted()
            })
            .borders(Borders::ALL)
            .border_style(theme::style_border(parent_focused));

        let parent_display = match self.selected_parent_index {
            None => "< None >".to_string(),
            Some(idx) => format!("< {} >", self.parent_groups[idx].1),
        };

        let parent_style = if parent_focused {
            theme::style_accent()
        } else {
            theme::style_muted()
        };
        frame.render_widget(
            Paragraph::new(Span::styled(parent_display, parent_style)).block(parent_block),
            chunks[1],
        );

        // Hints
        let hints = Paragraph::new(Line::from(vec![
            Span::styled("Tab", theme::style_accent()),
            Span::raw(" next  "),
            Span::styled("Ctrl+S", theme::style_accent()),
            Span::raw(" save  "),
            Span::styled("Esc", theme::style_accent()),
            Span::raw(" cancel"),
        ]))
        .style(theme::style_muted());
        frame.render_widget(hints, chunks[2]);
    }
}
