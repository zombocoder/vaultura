use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use uuid::Uuid;

use crate::core::models::{Group, Item};
use crate::ui::panels::details_panel::DetailsPanel;
use crate::ui::panels::groups_panel::GroupsPanel;
use crate::ui::panels::items_panel::ItemsPanel;
use crate::ui::theme;
use crate::ui::{Action, Component};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pane {
    Groups,
    Items,
    Details,
}

pub struct MainScreen {
    pub groups_panel: GroupsPanel,
    pub items_panel: ItemsPanel,
    pub details_panel: DetailsPanel,
    active_pane: Pane,
    status_message: Option<(String, Instant)>,
}

const STATUS_DISPLAY_SECS: u64 = 3;

impl Default for MainScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl MainScreen {
    pub fn new() -> Self {
        let mut groups_panel = GroupsPanel::new();
        groups_panel.set_focused(true);

        Self {
            groups_panel,
            items_panel: ItemsPanel::new(),
            details_panel: DetailsPanel::new(),
            active_pane: Pane::Groups,
            status_message: None,
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Clear the status message if it has expired.
    pub fn tick(&mut self) {
        if let Some((_, set_at)) = &self.status_message {
            if set_at.elapsed().as_secs() >= STATUS_DISPLAY_SECS {
                self.status_message = None;
            }
        }
    }

    pub fn update_groups(&mut self, groups: &[Group]) {
        self.groups_panel.update_groups(groups);
    }

    pub fn update_items(&mut self, items: &[&Item]) {
        self.items_panel.update_items(items);
    }

    pub fn update_details(&mut self, item: Option<&Item>, group_name: &str) {
        self.details_panel.set_item(item, group_name);
    }

    pub fn selected_group_id(&self) -> Option<Uuid> {
        self.groups_panel.selected_group_id()
    }

    pub fn selected_item_id(&self) -> Option<Uuid> {
        self.items_panel.selected_item_id()
    }

    pub fn selected_group_name(&self) -> Option<String> {
        self.groups_panel.selected_group_name()
    }

    fn cycle_pane_forward(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Groups => Pane::Items,
            Pane::Items => Pane::Details,
            Pane::Details => Pane::Groups,
        };
        self.update_focus();
    }

    fn cycle_pane_backward(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Groups => Pane::Details,
            Pane::Items => Pane::Groups,
            Pane::Details => Pane::Items,
        };
        self.update_focus();
    }

    fn update_focus(&mut self) {
        self.groups_panel
            .set_focused(self.active_pane == Pane::Groups);
        self.items_panel
            .set_focused(self.active_pane == Pane::Items);
        self.details_panel
            .set_focused(self.active_pane == Pane::Details);
    }
}

impl Component for MainScreen {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        // Global keys
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Action::Quit,
            (KeyCode::Char('l'), KeyModifiers::CONTROL) => return Action::Lock,
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => return Action::Save,
            (KeyCode::Char('q'), KeyModifiers::NONE) => {
                // Don't quit if search is active or in details
                if !self.items_panel.is_search_active() {
                    return Action::Quit;
                }
            }
            _ => {}
        }

        // Tab navigation (only when search not active)
        if !self.items_panel.is_search_active() {
            match key.code {
                KeyCode::Tab => {
                    self.cycle_pane_forward();
                    return Action::None;
                }
                KeyCode::BackTab => {
                    self.cycle_pane_backward();
                    return Action::None;
                }
                _ => {}
            }
        }

        // Delegate to active panel
        match self.active_pane {
            Pane::Groups => self.groups_panel.handle_key(key),
            Pane::Items => self.items_panel.handle_key(key),
            Pane::Details => self.details_panel.handle_key(key),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([
            Constraint::Min(1),    // Main area
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        // 3-pane layout: Groups 20% | Items 35% | Details 45%
        let panes = Layout::horizontal([
            Constraint::Percentage(20),
            Constraint::Percentage(35),
            Constraint::Percentage(45),
        ])
        .split(chunks[0]);

        self.groups_panel.render(frame, panes[0]);
        self.items_panel.render(frame, panes[1]);
        self.details_panel.render(frame, panes[2]);

        // Status bar
        let status_text = if let Some((ref msg, _)) = self.status_message {
            Line::from(vec![
                Span::styled(" ", theme::style_default()),
                Span::raw(msg.as_str()),
            ])
        } else {
            Line::from(vec![
                Span::styled(" Tab", theme::style_accent()),
                Span::raw(" switch pane  "),
                Span::styled("n", theme::style_accent()),
                Span::raw(" new item  "),
                Span::styled("g", theme::style_accent()),
                Span::raw(" new group  "),
                Span::styled("/", theme::style_accent()),
                Span::raw(" search  "),
                Span::styled("Ctrl+L", theme::style_accent()),
                Span::raw(" lock  "),
                Span::styled("q", theme::style_accent()),
                Span::raw(" quit"),
            ])
        };

        let status = Paragraph::new(status_text).style(theme::style_muted());
        frame.render_widget(status, chunks[1]);
    }
}
