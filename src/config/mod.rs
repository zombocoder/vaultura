use std::fs;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{Result, VaulturaError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub vault_path: PathBuf,
    pub auto_lock_secs: u64,
    pub clipboard_clear_secs: u64,
    pub kdf_memory_cost_kib: u32,
    pub kdf_time_cost: u32,
    pub kdf_parallelism: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            vault_path: default_vault_path(),
            auto_lock_secs: 300,
            clipboard_clear_secs: 30,
            kdf_memory_cost_kib: 65536,
            kdf_time_cost: 3,
            kdf_parallelism: 4,
        }
    }
}

impl AppConfig {
    pub fn kdf_params(&self) -> crate::core::models::KdfParams {
        crate::core::models::KdfParams {
            memory_cost_kib: self.kdf_memory_cost_kib,
            time_cost: self.kdf_time_cost,
            parallelism: self.kdf_parallelism,
        }
    }

    pub fn load() -> Result<Self> {
        let path = config_file_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = config_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let config: AppConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Err(VaulturaError::Config(format!(
                "Config file not found: {}",
                path.display()
            )))
        }
    }
}

fn config_file_path() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("", "", "vaultura") {
        dirs.config_dir().join("config.toml")
    } else {
        PathBuf::from("vaultura.toml")
    }
}

fn default_vault_path() -> PathBuf {
    if let Some(dirs) = ProjectDirs::from("", "", "vaultura") {
        dirs.data_dir().join("vault.vltr")
    } else {
        PathBuf::from("vault.vltr")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");

        let config = AppConfig {
            vault_path: PathBuf::from("/tmp/test.vltr"),
            auto_lock_secs: 120,
            clipboard_clear_secs: 15,
            kdf_memory_cost_kib: 32768,
            kdf_time_cost: 2,
            kdf_parallelism: 2,
        };

        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&path, &content).unwrap();
        let loaded = AppConfig::load_from(&path).unwrap();

        assert_eq!(loaded.vault_path, config.vault_path);
        assert_eq!(loaded.auto_lock_secs, config.auto_lock_secs);
        assert_eq!(loaded.clipboard_clear_secs, config.clipboard_clear_secs);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.auto_lock_secs, 300);
        assert_eq!(config.clipboard_clear_secs, 30);
        assert_eq!(config.kdf_memory_cost_kib, 65536);
    }

    #[test]
    fn test_kdf_params_from_config() {
        let config = AppConfig::default();
        let params = config.kdf_params();
        assert_eq!(params.memory_cost_kib, 65536);
        assert_eq!(params.time_cost, 3);
        assert_eq!(params.parallelism, 4);
    }
}
