use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ratatui::Frame;
use uuid::Uuid;

use crate::clipboard::ClipboardManager;
use crate::config::AppConfig;
use crate::core::vault_service::VaultService;
use crate::ui::modals::confirm_dialog::ConfirmDialog;
use crate::ui::modals::group_form::GroupForm;
use crate::ui::modals::item_form::ItemForm;
use crate::ui::modals::password_generator_modal::PasswordGeneratorModal;
use crate::ui::screens::lock_screen::LockScreen;
use crate::ui::screens::main_screen::MainScreen;
use crate::ui::{Action, Component};

const TICK_RATE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    Lock,
    Main,
}

enum Modal {
    None,
    ItemForm(ItemForm),
    GroupForm(GroupForm),
    Confirm(ConfirmDialog),
    PasswordGenerator(PasswordGeneratorModal),
}

pub struct App {
    vault_service: VaultService,
    clipboard: ClipboardManager,
    config: AppConfig,
    lock_screen: LockScreen,
    main_screen: MainScreen,
    current_screen: Screen,
    modal: Modal,
    /// Stashed item form while the password generator is open on top of it.
    stashed_item_form: Option<ItemForm>,
    running: bool,
    last_activity: Instant,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let kdf_params = config.kdf_params();
        let vault_path = config.vault_path.clone();
        let vault_exists = vault_path.exists();
        let clipboard_secs = config.clipboard_clear_secs;

        Self {
            vault_service: VaultService::new(vault_path, kdf_params),
            clipboard: ClipboardManager::new(clipboard_secs),
            config,
            lock_screen: LockScreen::new(vault_exists),
            main_screen: MainScreen::new(),
            current_screen: Screen::Lock,
            modal: Modal::None,
            stashed_item_form: None,
            running: true,
            last_activity: Instant::now(),
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            // Expire status messages
            self.main_screen.tick();

            // Auto-lock check
            if self.current_screen == Screen::Main
                && self.config.auto_lock_secs > 0
                && self.last_activity.elapsed() > Duration::from_secs(self.config.auto_lock_secs)
            {
                self.handle_action(Action::Lock);
            }

            if event::poll(TICK_RATE)? {
                if let Event::Key(key) = event::read()? {
                    self.last_activity = Instant::now();
                    let action = self.handle_input(key);
                    self.handle_action(action);
                }
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        match self.current_screen {
            Screen::Lock => self.lock_screen.render(frame, area),
            Screen::Main => {
                self.main_screen.render(frame, area);

                // Render modal overlay if present
                match &self.modal {
                    Modal::None => {}
                    Modal::ItemForm(form) => form.render(frame, area),
                    Modal::GroupForm(form) => form.render(frame, area),
                    Modal::Confirm(dialog) => dialog.render(frame, area),
                    Modal::PasswordGenerator(gen) => gen.render(frame, area),
                }
            }
        }
    }

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Action {
        // Modal gets input first
        match &mut self.modal {
            Modal::None => {}
            Modal::ItemForm(form) => return form.handle_key(key),
            Modal::GroupForm(form) => return form.handle_key(key),
            Modal::Confirm(dialog) => return dialog.handle_key(key),
            Modal::PasswordGenerator(gen) => return gen.handle_key(key),
        }

        match self.current_screen {
            Screen::Lock => self.lock_screen.handle_key(key),
            Screen::Main => self.main_screen.handle_key(key),
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Quit => {
                if self.vault_service.is_dirty() {
                    let _ = self.vault_service.save();
                }
                self.running = false;
            }
            Action::Lock => {
                if self.vault_service.is_dirty() {
                    let _ = self.vault_service.save();
                }
                self.vault_service.lock();
                self.current_screen = Screen::Lock;
                self.lock_screen.clear();
                self.lock_screen.set_vault_exists(true);
                self.modal = Modal::None;
                self.stashed_item_form = None;
                self.main_screen = MainScreen::new();
            }
            Action::Save => match self.vault_service.save() {
                Ok(()) => self.main_screen.set_status("Saved".to_string()),
                Err(e) => self.main_screen.set_status(format!("Save failed: {e}")),
            },
            Action::CreateVault(password) => {
                // Ensure parent directory exists
                if let Some(parent) = self.vault_service.vault_path().parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match self.vault_service.create(&password) {
                    Ok(()) => {
                        self.current_screen = Screen::Main;
                        self.refresh_ui();
                    }
                    Err(e) => self.lock_screen.set_error(format!("{e}")),
                }
            }
            Action::UnlockVault(password) => match self.vault_service.unlock(&password) {
                Ok(()) => {
                    self.current_screen = Screen::Main;
                    self.refresh_ui();
                }
                Err(e) => self.lock_screen.set_error(format!("{e}")),
            },
            Action::SelectGroup(group_id) => {
                self.refresh_items(group_id);
            }
            Action::SelectItem(item_id) => {
                self.refresh_details(item_id);
            }
            Action::CreateItem(draft) => match self.vault_service.create_item(draft) {
                Ok(_id) => {
                    self.modal = Modal::None;
                    self.auto_save();
                    self.refresh_ui();
                    self.main_screen.set_status("Item created".to_string());
                }
                Err(e) => self.main_screen.set_status(format!("Error: {e}")),
            },
            Action::UpdateItem(id, draft) => match self.vault_service.update_item(id, draft) {
                Ok(()) => {
                    self.modal = Modal::None;
                    self.auto_save();
                    self.refresh_ui();
                    self.main_screen.set_status("Item updated".to_string());
                }
                Err(e) => self.main_screen.set_status(format!("Error: {e}")),
            },
            Action::DeleteItem(id) => match self.vault_service.delete_item(id) {
                Ok(()) => {
                    self.modal = Modal::None;
                    self.auto_save();
                    self.main_screen.details_panel.clear();
                    self.refresh_ui();
                    self.main_screen.set_status("Item deleted".to_string());
                }
                Err(e) => self.main_screen.set_status(format!("Error: {e}")),
            },
            Action::CreateGroup(name, parent_id) => {
                match self.vault_service.create_group(name, parent_id) {
                    Ok(_id) => {
                        self.modal = Modal::None;
                        self.auto_save();
                        self.refresh_ui();
                        self.main_screen.set_status("Group created".to_string());
                    }
                    Err(e) => self.main_screen.set_status(format!("Error: {e}")),
                }
            }
            Action::UpdateGroup(id, name, parent_id) => {
                match self.vault_service.update_group(id, name, parent_id) {
                    Ok(()) => {
                        self.modal = Modal::None;
                        self.auto_save();
                        self.refresh_ui();
                        self.main_screen.set_status("Group updated".to_string());
                    }
                    Err(e) => self.main_screen.set_status(format!("Error: {e}")),
                }
            }
            Action::DeleteGroup(id) => match self.vault_service.delete_group(id) {
                Ok(()) => {
                    self.modal = Modal::None;
                    self.auto_save();
                    self.refresh_ui();
                    self.main_screen.set_status("Group deleted".to_string());
                }
                Err(e) => self.main_screen.set_status(format!("Error: {e}")),
            },
            Action::CopyPassword(id) => {
                if let Ok(item) = self.vault_service.get_item(id) {
                    let pw = item.password.clone();
                    match self.clipboard.copy_and_clear(&pw) {
                        Ok(()) => self.main_screen.set_status(format!(
                            "Password copied (clears in {}s)",
                            self.config.clipboard_clear_secs
                        )),
                        Err(e) => self.main_screen.set_status(format!("Clipboard error: {e}")),
                    }
                }
            }
            Action::CopyUsername(id) => {
                if let Ok(item) = self.vault_service.get_item(id) {
                    let un = item.username.clone();
                    match self.clipboard.copy_and_clear(&un) {
                        Ok(()) => self.main_screen.set_status(format!(
                            "Username copied (clears in {}s)",
                            self.config.clipboard_clear_secs
                        )),
                        Err(e) => self.main_screen.set_status(format!("Clipboard error: {e}")),
                    }
                }
            }
            Action::SetSearchQuery(query) => {
                let group_id = self.main_screen.selected_group_id();
                if let Ok(items) = self.vault_service.search_in_group(&query, group_id) {
                    self.main_screen.update_items(&items);
                }
            }
            Action::ClearSearch => {
                let group_id = self.main_screen.selected_group_id();
                self.refresh_items(group_id);
            }
            Action::OpenNewItemForm => {
                if let Ok(groups) = self.vault_service.groups() {
                    let default_group = self.main_screen.selected_group_id();
                    let form = ItemForm::new_create(groups, default_group);
                    self.modal = Modal::ItemForm(form);
                }
            }
            Action::OpenEditItemForm(id) => {
                if let (Ok(item), Ok(groups)) =
                    (self.vault_service.get_item(id), self.vault_service.groups())
                {
                    let item = item.clone();
                    let groups = groups.to_vec();
                    let form = ItemForm::new_edit(&item, &groups);
                    self.modal = Modal::ItemForm(form);
                }
            }
            Action::OpenDeleteConfirm(id) => {
                let name = self
                    .vault_service
                    .get_item(id)
                    .map(|i| i.title.clone())
                    .unwrap_or_default();
                let dialog =
                    ConfirmDialog::new(format!("Delete item \"{name}\"?"), Action::DeleteItem(id));
                self.modal = Modal::Confirm(dialog);
            }
            Action::OpenNewGroupForm => {
                if let Ok(groups) = self.vault_service.groups() {
                    let groups = groups.to_vec();
                    self.modal = Modal::GroupForm(GroupForm::new_create(&groups));
                }
            }
            Action::OpenEditGroupForm(id) => {
                if let Ok(groups) = self.vault_service.groups() {
                    let groups = groups.to_vec();
                    if let Some(group) = groups.iter().find(|g| g.id == id) {
                        self.modal = Modal::GroupForm(GroupForm::new_edit(group, &groups));
                    }
                }
            }
            Action::OpenDeleteGroupConfirm(id) => {
                if let Ok(groups) = self.vault_service.groups() {
                    let name = groups
                        .iter()
                        .find(|g| g.id == id)
                        .map(|g| g.name.clone())
                        .unwrap_or_default();
                    let dialog = ConfirmDialog::new(
                        format!("Delete group \"{name}\"?"),
                        Action::DeleteGroup(id),
                    );
                    self.modal = Modal::Confirm(dialog);
                }
            }
            Action::OpenPasswordGenerator => {
                let for_item_form = matches!(self.modal, Modal::ItemForm(_));
                if for_item_form {
                    // Stash the item form so we can restore it after the generator closes.
                    let old_modal = std::mem::replace(
                        &mut self.modal,
                        Modal::PasswordGenerator(PasswordGeneratorModal::new()),
                    );
                    if let Modal::ItemForm(form) = old_modal {
                        self.stashed_item_form = Some(form);
                    }
                } else {
                    self.modal = Modal::PasswordGenerator(PasswordGeneratorModal::new());
                }
            }
            Action::UseGeneratedPassword => {
                // Extract generated password, restore stashed item form with it.
                if let Modal::PasswordGenerator(ref gen) = self.modal {
                    let pw = gen.generated_password().to_string();
                    if let Some(mut form) = self.stashed_item_form.take() {
                        form.set_password(pw);
                        self.modal = Modal::ItemForm(form);
                    } else {
                        // No item form stashed — copy to clipboard instead.
                        let _ = self.clipboard.copy_and_clear(&pw);
                        self.main_screen.set_status(format!(
                            "Password copied (clears in {}s)",
                            self.config.clipboard_clear_secs
                        ));
                        self.modal = Modal::None;
                    }
                }
            }
            Action::CloseModal => {
                // Esc / cancel: restore stashed form without applying password.
                if let Some(form) = self.stashed_item_form.take() {
                    self.modal = Modal::ItemForm(form);
                } else {
                    self.modal = Modal::None;
                }
            }
            Action::SetStatus(msg) => {
                self.main_screen.set_status(msg);
            }
        }
    }

    fn refresh_ui(&mut self) {
        if let Ok(groups) = self.vault_service.groups() {
            let groups = groups.to_vec();
            self.main_screen.update_groups(&groups);
        }
        let group_id = self.main_screen.selected_group_id();
        self.refresh_items(group_id);
    }

    fn refresh_items(&mut self, group_id: Option<Uuid>) {
        let query = self.main_screen.items_panel.search_query().to_string();
        let items = if query.is_empty() {
            self.vault_service
                .items_in_group(group_id)
                .unwrap_or_default()
        } else {
            self.vault_service
                .search_in_group(&query, group_id)
                .unwrap_or_default()
        };
        self.main_screen.update_items(&items);

        // Auto-select first item
        let first_id = self.main_screen.selected_item_id();
        self.refresh_details(first_id);
    }

    fn refresh_details(&mut self, item_id: Option<Uuid>) {
        if let Some(id) = item_id {
            if let Ok(item) = self.vault_service.get_item(id) {
                let item = item.clone();
                let group_name = item
                    .group_id
                    .and_then(|gid| {
                        self.vault_service.groups().ok().and_then(|groups| {
                            groups.iter().find(|g| g.id == gid).map(|g| g.name.clone())
                        })
                    })
                    .unwrap_or_else(|| "None".to_string());
                self.main_screen.update_details(Some(&item), &group_name);
            }
        } else {
            self.main_screen.update_details(None, "");
        }
    }

    fn auto_save(&mut self) {
        if self.vault_service.is_dirty() {
            if let Err(e) = self.vault_service.save() {
                self.main_screen
                    .set_status(format!("Auto-save failed: {e}"));
            }
        }
    }
}
