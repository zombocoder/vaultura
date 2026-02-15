use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KdfParams {
    pub memory_cost_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost_kib: 65536, // 64 MB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

impl KdfParams {
    /// Fast parameters for testing only.
    #[cfg(test)]
    pub fn test_params() -> Self {
        Self {
            memory_cost_kib: 1024, // 1 MB
            time_cost: 1,
            parallelism: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CipherParams {
    pub nonce: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultMeta {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Default for VaultMeta {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: 1,
            created_at: now,
            modified_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: String, parent_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            parent_id,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasswordHistoryEntry {
    pub password: String,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Item {
    pub id: Uuid,
    pub group_id: Option<Uuid>,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub password_history: Vec<PasswordHistoryEntry>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

impl Item {
    pub fn new(title: String, group_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            group_id,
            title,
            username: String::new(),
            password: String::new(),
            url: String::new(),
            notes: String::new(),
            tags: Vec::new(),
            password_history: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct VaultPayload {
    pub meta: VaultMeta,
    pub groups: Vec<Group>,
    pub items: Vec<Item>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_params_roundtrip() {
        let params = KdfParams::default();
        let encoded = bincode::serialize(&params).unwrap();
        let decoded: KdfParams = bincode::deserialize(&encoded).unwrap();
        assert_eq!(params, decoded);
    }

    #[test]
    fn test_group_roundtrip() {
        let group = Group::new("Test Group".to_string(), None);
        let encoded = bincode::serialize(&group).unwrap();
        let decoded: Group = bincode::deserialize(&encoded).unwrap();
        assert_eq!(group, decoded);
    }

    #[test]
    fn test_item_roundtrip() {
        let mut item = Item::new("Test Item".to_string(), None);
        item.username = "user@example.com".to_string();
        item.password = "secret123".to_string();
        item.url = "https://example.com".to_string();
        item.notes = "Some notes".to_string();
        item.tags = vec!["tag1".to_string(), "tag2".to_string()];
        item.password_history.push(PasswordHistoryEntry {
            password: "old_pass".to_string(),
            changed_at: Utc::now(),
        });
        let encoded = bincode::serialize(&item).unwrap();
        let decoded: Item = bincode::deserialize(&encoded).unwrap();
        assert_eq!(item, decoded);
    }

    #[test]
    fn test_vault_payload_roundtrip() {
        let mut payload = VaultPayload::default();
        payload.groups.push(Group::new("Group1".to_string(), None));
        payload
            .items
            .push(Item::new("Item1".to_string(), Some(payload.groups[0].id)));
        let encoded = bincode::serialize(&payload).unwrap();
        let decoded: VaultPayload = bincode::deserialize(&encoded).unwrap();
        assert_eq!(payload, decoded);
    }

    #[test]
    fn test_vault_meta_roundtrip() {
        let meta = VaultMeta::default();
        let encoded = bincode::serialize(&meta).unwrap();
        let decoded: VaultMeta = bincode::deserialize(&encoded).unwrap();
        assert_eq!(meta, decoded);
    }

    #[test]
    fn test_cipher_params_roundtrip() {
        let params = CipherParams {
            nonce: vec![1, 2, 3, 4, 5],
        };
        let encoded = bincode::serialize(&params).unwrap();
        let decoded: CipherParams = bincode::deserialize(&encoded).unwrap();
        assert_eq!(params, decoded);
    }
}
