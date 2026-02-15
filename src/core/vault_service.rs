use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use crate::core::models::{Group, Item, KdfParams, PasswordHistoryEntry, VaultPayload};
use crate::error::{Result, VaulturaError};
use crate::storage::vault_file;

/// Draft for creating or editing items (used by the UI layer).
#[derive(Debug, Clone, Default)]
pub struct ItemDraft {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub group_id: Option<Uuid>,
}

pub struct VaultService {
    vault_path: PathBuf,
    password: Option<String>,
    kdf_params: KdfParams,
    payload: Option<VaultPayload>,
    dirty: bool,
}

impl VaultService {
    pub fn new(vault_path: PathBuf, kdf_params: KdfParams) -> Self {
        Self {
            vault_path,
            password: None,
            kdf_params,
            payload: None,
            dirty: false,
        }
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    pub fn vault_exists(&self) -> bool {
        self.vault_path.exists()
    }

    pub fn is_unlocked(&self) -> bool {
        self.payload.is_some()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Create a new vault with an empty payload.
    pub fn create(&mut self, password: &str) -> Result<()> {
        vault_file::create_vault(&self.vault_path, password, &self.kdf_params)?;
        self.password = Some(password.to_string());
        self.payload = Some(VaultPayload::default());
        self.dirty = false;
        Ok(())
    }

    /// Unlock an existing vault.
    pub fn unlock(&mut self, password: &str) -> Result<()> {
        let (payload, kdf_params) = vault_file::read_vault(&self.vault_path, password)?;
        self.password = Some(password.to_string());
        self.kdf_params = kdf_params;
        self.payload = Some(payload);
        self.dirty = false;
        Ok(())
    }

    /// Lock the vault, wiping decrypted data from memory.
    pub fn lock(&mut self) {
        self.payload = None;
        self.password = None;
        self.dirty = false;
    }

    /// Save the current payload to disk.
    pub fn save(&mut self) -> Result<()> {
        let password = self
            .password
            .as_ref()
            .ok_or(VaulturaError::VaultLocked)?
            .clone();
        let payload = self.payload.as_ref().ok_or(VaulturaError::VaultLocked)?;
        vault_file::write_vault(&self.vault_path, &password, &self.kdf_params, payload)?;
        self.dirty = false;
        Ok(())
    }

    fn payload(&self) -> Result<&VaultPayload> {
        self.payload.as_ref().ok_or(VaulturaError::VaultLocked)
    }

    fn payload_mut(&mut self) -> Result<&mut VaultPayload> {
        self.payload.as_mut().ok_or(VaulturaError::VaultLocked)
    }

    // --- Groups ---

    pub fn groups(&self) -> Result<&[Group]> {
        Ok(&self.payload()?.groups)
    }

    pub fn create_group(&mut self, name: String, parent_id: Option<Uuid>) -> Result<Uuid> {
        let group = Group::new(name, parent_id);
        let id = group.id;
        self.payload_mut()?.groups.push(group);
        self.dirty = true;
        Ok(id)
    }

    pub fn update_group(&mut self, id: Uuid, name: String, parent_id: Option<Uuid>) -> Result<()> {
        let payload = self.payload_mut()?;
        let group = payload
            .groups
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or(VaulturaError::GroupNotFound(id))?;
        group.name = name;
        group.parent_id = parent_id;
        self.dirty = true;
        Ok(())
    }

    pub fn delete_group(&mut self, id: Uuid) -> Result<()> {
        let payload = self.payload_mut()?;
        let existed = payload.groups.len();
        payload.groups.retain(|g| g.id != id);
        if payload.groups.len() == existed {
            return Err(VaulturaError::GroupNotFound(id));
        }
        // Ungroup items that belonged to this group
        for item in &mut payload.items {
            if item.group_id == Some(id) {
                item.group_id = None;
            }
        }
        self.dirty = true;
        Ok(())
    }

    // --- Items ---

    pub fn items(&self) -> Result<&[Item]> {
        Ok(&self.payload()?.items)
    }

    pub fn items_in_group(&self, group_id: Option<Uuid>) -> Result<Vec<&Item>> {
        let payload = self.payload()?;
        match group_id {
            None => Ok(payload.items.iter().collect()),
            Some(gid) => Ok(payload
                .items
                .iter()
                .filter(|i| i.group_id == Some(gid))
                .collect()),
        }
    }

    pub fn get_item(&self, id: Uuid) -> Result<&Item> {
        self.payload()?
            .items
            .iter()
            .find(|i| i.id == id)
            .ok_or(VaulturaError::ItemNotFound(id))
    }

    pub fn create_item(&mut self, draft: ItemDraft) -> Result<Uuid> {
        let mut item = Item::new(draft.title, draft.group_id);
        item.username = draft.username;
        item.password = draft.password;
        item.url = draft.url;
        item.notes = draft.notes;
        item.tags = draft.tags;
        let id = item.id;
        self.payload_mut()?.items.push(item);
        self.dirty = true;
        Ok(id)
    }

    pub fn update_item(&mut self, id: Uuid, draft: ItemDraft) -> Result<()> {
        let payload = self.payload_mut()?;
        let item = payload
            .items
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or(VaulturaError::ItemNotFound(id))?;

        // Track password history if password changed
        if item.password != draft.password && !item.password.is_empty() {
            item.password_history.push(PasswordHistoryEntry {
                password: item.password.clone(),
                changed_at: Utc::now(),
            });
        }

        item.title = draft.title;
        item.username = draft.username;
        item.password = draft.password;
        item.url = draft.url;
        item.notes = draft.notes;
        item.tags = draft.tags;
        item.group_id = draft.group_id;
        item.modified_at = Utc::now();
        self.dirty = true;
        Ok(())
    }

    pub fn delete_item(&mut self, id: Uuid) -> Result<()> {
        let payload = self.payload_mut()?;
        let existed = payload.items.len();
        payload.items.retain(|i| i.id != id);
        if payload.items.len() == existed {
            return Err(VaulturaError::ItemNotFound(id));
        }
        self.dirty = true;
        Ok(())
    }

    /// Case-insensitive multi-token AND search across title, username, url, notes, and tags.
    pub fn search(&self, query: &str) -> Result<Vec<&Item>> {
        let payload = self.payload()?;
        if query.is_empty() {
            return Ok(payload.items.iter().collect());
        }

        let tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();

        Ok(payload
            .items
            .iter()
            .filter(|item| {
                let searchable = format!(
                    "{} {} {} {} {}",
                    item.title,
                    item.username,
                    item.url,
                    item.notes,
                    item.tags.join(" ")
                )
                .to_lowercase();

                tokens
                    .iter()
                    .all(|token| searchable.contains(token.as_str()))
            })
            .collect())
    }

    /// Search within a specific group.
    pub fn search_in_group(&self, query: &str, group_id: Option<Uuid>) -> Result<Vec<&Item>> {
        let results = self.search(query)?;
        match group_id {
            None => Ok(results),
            Some(gid) => Ok(results
                .into_iter()
                .filter(|i| i.group_id == Some(gid))
                .collect()),
        }
    }

    // --- Import/Export ---

    pub fn export(&self, path: &Path, password: &str) -> Result<()> {
        let payload = self.payload()?;
        vault_file::export_vault(path, password, &self.kdf_params, payload)
    }

    pub fn import(&mut self, path: &Path, password: &str) -> Result<usize> {
        let imported = vault_file::import_vault(path, password)?;
        let payload = self.payload_mut()?;
        let count = imported.items.len() + imported.groups.len();

        for group in imported.groups {
            if !payload.groups.iter().any(|g| g.id == group.id) {
                payload.groups.push(group);
            }
        }
        for item in imported.items {
            if !payload.items.iter().any(|i| i.id == item.id) {
                payload.items.push(item);
            }
        }

        self.dirty = true;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_params() -> KdfParams {
        KdfParams {
            memory_cost_kib: 1024,
            time_cost: 1,
            parallelism: 1,
        }
    }

    fn setup() -> (TempDir, VaultService) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let mut svc = VaultService::new(path, test_params());
        svc.create("password").unwrap();
        (dir, svc)
    }

    #[test]
    fn test_create_and_unlock() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let mut svc = VaultService::new(path.clone(), test_params());

        assert!(!svc.vault_exists());
        svc.create("password").unwrap();
        assert!(svc.vault_exists());
        assert!(svc.is_unlocked());

        svc.lock();
        assert!(!svc.is_unlocked());

        svc.unlock("password").unwrap();
        assert!(svc.is_unlocked());
    }

    #[test]
    fn test_wrong_password_unlock() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let mut svc = VaultService::new(path, test_params());
        svc.create("correct").unwrap();
        svc.lock();

        let result = svc.unlock("wrong");
        assert!(matches!(result, Err(VaulturaError::WrongPassword)));
    }

    #[test]
    fn test_crud_groups() {
        let (_dir, mut svc) = setup();

        let gid = svc.create_group("Work".to_string(), None).unwrap();
        assert_eq!(svc.groups().unwrap().len(), 1);
        assert_eq!(svc.groups().unwrap()[0].name, "Work");

        svc.update_group(gid, "Personal".to_string(), None).unwrap();
        assert_eq!(svc.groups().unwrap()[0].name, "Personal");

        svc.delete_group(gid).unwrap();
        assert!(svc.groups().unwrap().is_empty());
    }

    #[test]
    fn test_crud_items() {
        let (_dir, mut svc) = setup();

        let draft = ItemDraft {
            title: "GitHub".to_string(),
            username: "user@example.com".to_string(),
            password: "secret".to_string(),
            url: "https://github.com".to_string(),
            notes: "My GitHub account".to_string(),
            tags: vec!["dev".to_string()],
            group_id: None,
        };

        let item_id = svc.create_item(draft).unwrap();
        assert_eq!(svc.items().unwrap().len(), 1);

        let item = svc.get_item(item_id).unwrap();
        assert_eq!(item.title, "GitHub");
        assert_eq!(item.username, "user@example.com");

        let update = ItemDraft {
            title: "GitHub Updated".to_string(),
            username: "new@example.com".to_string(),
            password: "new_secret".to_string(),
            url: "https://github.com".to_string(),
            notes: "Updated notes".to_string(),
            tags: vec!["dev".to_string(), "vcs".to_string()],
            group_id: None,
        };
        svc.update_item(item_id, update).unwrap();

        let item = svc.get_item(item_id).unwrap();
        assert_eq!(item.title, "GitHub Updated");
        assert_eq!(item.password_history.len(), 1);
        assert_eq!(item.password_history[0].password, "secret");

        svc.delete_item(item_id).unwrap();
        assert!(svc.items().unwrap().is_empty());
    }

    #[test]
    fn test_delete_group_ungroups_items() {
        let (_dir, mut svc) = setup();

        let gid = svc.create_group("Work".to_string(), None).unwrap();
        let draft = ItemDraft {
            title: "Item".to_string(),
            group_id: Some(gid),
            ..Default::default()
        };
        let item_id = svc.create_item(draft).unwrap();

        svc.delete_group(gid).unwrap();
        let item = svc.get_item(item_id).unwrap();
        assert_eq!(item.group_id, None);
    }

    #[test]
    fn test_items_in_group() {
        let (_dir, mut svc) = setup();

        let gid = svc.create_group("Work".to_string(), None).unwrap();
        svc.create_item(ItemDraft {
            title: "In group".to_string(),
            group_id: Some(gid),
            ..Default::default()
        })
        .unwrap();
        svc.create_item(ItemDraft {
            title: "No group".to_string(),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(svc.items_in_group(Some(gid)).unwrap().len(), 1);
        assert_eq!(svc.items_in_group(None).unwrap().len(), 2);
    }

    #[test]
    fn test_search() {
        let (_dir, mut svc) = setup();

        svc.create_item(ItemDraft {
            title: "GitHub".to_string(),
            username: "user@example.com".to_string(),
            tags: vec!["dev".to_string()],
            ..Default::default()
        })
        .unwrap();
        svc.create_item(ItemDraft {
            title: "Gmail".to_string(),
            username: "user@gmail.com".to_string(),
            tags: vec!["email".to_string()],
            ..Default::default()
        })
        .unwrap();

        assert_eq!(svc.search("git").unwrap().len(), 1);
        assert_eq!(svc.search("user").unwrap().len(), 2);
        assert_eq!(svc.search("dev").unwrap().len(), 1);
        assert_eq!(svc.search("GitHub user").unwrap().len(), 1);
        assert_eq!(svc.search("nonexistent").unwrap().len(), 0);
        assert_eq!(svc.search("").unwrap().len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let (_dir, mut svc) = setup();

        svc.create_item(ItemDraft {
            title: "GitHub".to_string(),
            ..Default::default()
        })
        .unwrap();

        assert_eq!(svc.search("github").unwrap().len(), 1);
        assert_eq!(svc.search("GITHUB").unwrap().len(), 1);
    }

    #[test]
    fn test_dirty_flag() {
        let (_dir, mut svc) = setup();
        assert!(!svc.is_dirty());

        svc.create_item(ItemDraft {
            title: "Test".to_string(),
            ..Default::default()
        })
        .unwrap();
        assert!(svc.is_dirty());

        svc.save().unwrap();
        assert!(!svc.is_dirty());
    }

    #[test]
    fn test_lock_unlock_persists() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let mut svc = VaultService::new(path.clone(), test_params());
        svc.create("password").unwrap();

        svc.create_item(ItemDraft {
            title: "Persistent".to_string(),
            ..Default::default()
        })
        .unwrap();
        svc.save().unwrap();
        svc.lock();

        svc.unlock("password").unwrap();
        assert_eq!(svc.items().unwrap().len(), 1);
        assert_eq!(svc.items().unwrap()[0].title, "Persistent");
    }

    #[test]
    fn test_export_import() {
        let dir = TempDir::new().unwrap();
        let path1 = dir.path().join("vault1.vault");
        let path2 = dir.path().join("vault2.vault");
        let export_path = dir.path().join("export.vault");

        let mut svc1 = VaultService::new(path1, test_params());
        svc1.create("pass1").unwrap();
        svc1.create_group("Group1".to_string(), None).unwrap();
        svc1.create_item(ItemDraft {
            title: "Item1".to_string(),
            ..Default::default()
        })
        .unwrap();
        svc1.save().unwrap();
        svc1.export(&export_path, "export_pass").unwrap();

        let mut svc2 = VaultService::new(path2, test_params());
        svc2.create("pass2").unwrap();
        let count = svc2.import(&export_path, "export_pass").unwrap();
        assert_eq!(count, 2); // 1 group + 1 item
        assert_eq!(svc2.items().unwrap().len(), 1);
        assert_eq!(svc2.groups().unwrap().len(), 1);
    }

    #[test]
    fn test_vault_locked_errors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let svc = VaultService::new(path, test_params());

        assert!(matches!(svc.items(), Err(VaulturaError::VaultLocked)));
        assert!(matches!(svc.groups(), Err(VaulturaError::VaultLocked)));
        assert!(matches!(svc.search("x"), Err(VaulturaError::VaultLocked)));
    }
}
