use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;
use uuid::Uuid;

use crate::core::models::Group;
use crate::ui::theme;
use crate::ui::{Action, Component};

pub struct GroupsPanel {
    /// None = "All Items" is the first entry, followed by group IDs.
    entries: Vec<Option<Uuid>>,
    group_names: Vec<String>,
    list_state: ListState,
    focused: bool,
}

impl Default for GroupsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupsPanel {
    pub fn new() -> Self {
        Self {
            entries: vec![None],
            group_names: vec!["All Items".to_string()],
            list_state: ListState::default().with_selected(Some(0)),
            focused: true,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn update_groups(&mut self, groups: &[Group]) {
        self.entries = vec![None];
        self.group_names = vec!["All Items".to_string()];
        for group in groups {
            self.entries.push(Some(group.id));
            self.group_names.push(group.name.clone());
        }
        // Clamp selection
        if let Some(sel) = self.list_state.selected() {
            if sel >= self.entries.len() {
                self.list_state
                    .select(Some(self.entries.len().saturating_sub(1)));
            }
        }
    }

    pub fn selected_group_id(&self) -> Option<Uuid> {
        self.list_state
            .selected()
            .and_then(|i| self.entries.get(i).copied())
            .flatten()
    }

    fn move_up(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 {
            self.list_state.select(Some(i - 1));
        }
    }

    fn move_down(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.entries.len() {
            self.list_state.select(Some(i + 1));
        }
    }

    pub fn selected_group_name(&self) -> Option<String> {
        self.list_state
            .selected()
            .and_then(|i| self.group_names.get(i))
            .cloned()
    }
}

impl Component for GroupsPanel {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if !self.focused {
            return Action::None;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_down();
                Action::SelectGroup(self.selected_group_id())
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_up();
                Action::SelectGroup(self.selected_group_id())
            }
            KeyCode::Enter => Action::SelectGroup(self.selected_group_id()),
            KeyCode::Char('g') => Action::OpenNewGroupForm,
            KeyCode::Char('G') => {
                if let Some(gid) = self.selected_group_id() {
                    Action::OpenEditGroupForm(gid)
                } else {
                    Action::None
                }
            }
            KeyCode::Char('D') => {
                if let Some(gid) = self.selected_group_id() {
                    Action::OpenDeleteGroupConfirm(gid)
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .group_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let prefix = if i == 0 { "📁 " } else { "  📂 " };
                ListItem::new(Line::raw(format!("{prefix}{name}")))
            })
            .collect();

        let block = Block::default()
            .title(" Groups ")
            .title_style(theme::style_title(self.focused))
            .borders(Borders::ALL)
            .border_style(theme::style_border(self.focused));

        let list = List::new(items)
            .block(block)
            .highlight_style(theme::style_selected())
            .highlight_symbol("▸ ");

        let mut state = self.list_state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }
}
