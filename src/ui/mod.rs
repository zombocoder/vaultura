pub mod app;
pub mod modals;
pub mod panels;
pub mod screens;
pub mod theme;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use uuid::Uuid;

use crate::core::vault_service::ItemDraft;

/// Actions emitted by UI components, dispatched by App.
#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    Lock,
    Save,

    // Vault lifecycle
    CreateVault(String),
    UnlockVault(String),

    // Navigation
    SelectGroup(Option<Uuid>),
    SelectItem(Option<Uuid>),

    // CRUD
    CreateItem(ItemDraft),
    UpdateItem(Uuid, ItemDraft),
    DeleteItem(Uuid),
    CreateGroup(String, Option<Uuid>),
    UpdateGroup(Uuid, String, Option<Uuid>),
    DeleteGroup(Uuid),

    // Clipboard
    CopyPassword(Uuid),
    CopyUsername(Uuid),

    // Search
    SetSearchQuery(String),
    ClearSearch,

    // Modals
    OpenNewItemForm,
    OpenEditItemForm(Uuid),
    OpenDeleteConfirm(Uuid),
    OpenNewGroupForm,
    OpenEditGroupForm(Uuid),
    OpenDeleteGroupConfirm(Uuid),
    OpenPasswordGenerator,
    UseGeneratedPassword,
    CloseModal,

    // Status
    SetStatus(String),

    // No-op
    None,
}

/// Trait implemented by all UI components (screens, panels, modals).
pub trait Component {
    fn handle_key(&mut self, key: KeyEvent) -> Action;
    fn render(&self, frame: &mut Frame, area: ratatui::layout::Rect);
}
