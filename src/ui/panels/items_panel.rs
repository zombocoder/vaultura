use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use uuid::Uuid;

use crate::core::models::Item;
use crate::ui::theme;
use crate::ui::{Action, Component};

pub struct ItemsPanel {
    items: Vec<(Uuid, String, String)>, // (id, title, username)
    list_state: ListState,
    focused: bool,
    search_active: bool,
    search_query: String,
}

impl Default for ItemsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemsPanel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            list_state: ListState::default(),
            focused: false,
            search_active: false,
            search_query: String::new(),
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_search_active(&self) -> bool {
        self.search_active
    }

    pub fn update_items(&mut self, items: &[&Item]) {
        self.items = items
            .iter()
            .map(|item| (item.id, item.title.clone(), item.username.clone()))
            .collect();
        // Clamp selection
        if self.items.is_empty() {
            self.list_state.select(None);
        } else if let Some(sel) = self.list_state.selected() {
            if sel >= self.items.len() {
                self.list_state.select(Some(self.items.len() - 1));
            }
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn selected_item_id(&self) -> Option<Uuid> {
        self.list_state
            .selected()
            .and_then(|i| self.items.get(i).map(|(id, _, _)| *id))
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    fn move_up(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i > 0 {
                self.list_state.select(Some(i - 1));
            }
        }
    }

    fn move_down(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i + 1 < self.items.len() {
                self.list_state.select(Some(i + 1));
            }
        }
    }
}

impl Component for ItemsPanel {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if !self.focused {
            return Action::None;
        }

        if self.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.search_active = false;
                    self.search_query.clear();
                    return Action::ClearSearch;
                }
                KeyCode::Enter => {
                    self.search_active = false;
                    return Action::None;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    return Action::SetSearchQuery(self.search_query.clone());
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    return Action::SetSearchQuery(self.search_query.clone());
                }
                _ => return Action::None,
            }
        }

        match key.code {
            KeyCode::Char('/') => {
                self.search_active = true;
                self.search_query.clear();
                Action::None
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                Action::SelectItem(self.selected_item_id())
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                Action::SelectItem(self.selected_item_id())
            }
            KeyCode::Enter => Action::SelectItem(self.selected_item_id()),
            KeyCode::Char('n') => Action::OpenNewItemForm,
            KeyCode::Char('e') => {
                if let Some(id) = self.selected_item_id() {
                    Action::OpenEditItemForm(id)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('d') => {
                if let Some(id) = self.selected_item_id() {
                    Action::OpenDeleteConfirm(id)
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Length(3), // Search bar
            Constraint::Min(1),    // Item list
        ])
        .split(area);

        // Search bar
        let search_block = Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(theme::style_border(self.search_active));

        let search_display = if self.search_active {
            Line::from(vec![
                Span::styled("/", theme::style_accent()),
                Span::raw(&self.search_query),
                Span::styled("█", theme::style_accent()),
            ])
        } else if self.search_query.is_empty() {
            Line::from(Span::styled("Press / to search...", theme::style_muted()))
        } else {
            Line::from(vec![
                Span::styled("/", theme::style_accent()),
                Span::raw(&self.search_query),
            ])
        };
        let search_para = Paragraph::new(search_display).block(search_block);
        frame.render_widget(search_para, chunks[0]);

        // Item list
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|(_, title, username)| {
                let line = if username.is_empty() {
                    Line::from(Span::raw(title.as_str()))
                } else {
                    Line::from(vec![
                        Span::raw(title.as_str()),
                        Span::styled(format!("  {username}"), theme::style_muted()),
                    ])
                };
                ListItem::new(line)
            })
            .collect();

        let list_block = Block::default()
            .title(format!(" Items ({}) ", self.items.len()))
            .title_style(theme::style_title(self.focused))
            .borders(Borders::ALL)
            .border_style(theme::style_border(self.focused));

        let list = List::new(items)
            .block(list_block)
            .highlight_style(theme::style_selected())
            .highlight_symbol("▸ ");

        let mut state = self.list_state.clone();
        frame.render_stateful_widget(list, chunks[1], &mut state);
    }
}
