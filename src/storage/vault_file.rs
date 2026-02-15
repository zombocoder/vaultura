use std::fs;
use std::io::Write;
use std::path::Path;

use crate::core::models::{KdfParams, VaultPayload};
use crate::crypto::{aead, kdf};
use crate::error::{Result, VaulturaError};
use crate::storage::format::{
    KDF_PARAMS_LENGTH, MAGIC, MIN_FILE_SIZE, NONCE_LENGTH, SALT_LENGTH, VERSION,
};

/// Create a new vault file at `path` with the given master password.
pub fn create_vault(path: &Path, password: &str, kdf_params: &KdfParams) -> Result<()> {
    let payload = VaultPayload::default();
    write_vault(path, password, kdf_params, &payload)
}

/// Write a vault payload to disk using atomic write (temp → fsync → rename).
pub fn write_vault(
    path: &Path,
    password: &str,
    kdf_params: &KdfParams,
    payload: &VaultPayload,
) -> Result<()> {
    let salt = kdf::generate_salt(SALT_LENGTH);
    let key = kdf::derive_key(password, &salt, kdf_params)?;

    let plaintext = bincode::serialize(payload)?;
    let (nonce, ciphertext) = aead::encrypt(&key, &plaintext)?;

    let mut data = Vec::new();
    data.extend_from_slice(MAGIC);
    data.extend_from_slice(&VERSION.to_le_bytes());
    data.extend_from_slice(&salt);
    write_kdf_params(&mut data, kdf_params);
    data.extend_from_slice(&nonce);
    data.extend_from_slice(&ciphertext);

    atomic_write(path, &data)
}

/// Read and decrypt a vault file, returning the payload.
pub fn read_vault(path: &Path, password: &str) -> Result<(VaultPayload, KdfParams)> {
    let data = fs::read(path)?;

    if data.len() < MIN_FILE_SIZE {
        return Err(VaulturaError::InvalidVaultFile {
            reason: "File too small".to_string(),
        });
    }

    let mut offset = 0;

    // Magic bytes
    if &data[offset..offset + 4] != MAGIC {
        return Err(VaulturaError::InvalidVaultFile {
            reason: "Invalid magic bytes".to_string(),
        });
    }
    offset += 4;

    // Version
    let version = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
    if version != VERSION {
        return Err(VaulturaError::InvalidVaultFile {
            reason: format!("Unsupported version: {version}"),
        });
    }
    offset += 4;

    // Salt
    let salt = &data[offset..offset + SALT_LENGTH];
    offset += SALT_LENGTH;

    // KDF params
    let kdf_params = read_kdf_params(&data[offset..offset + KDF_PARAMS_LENGTH]);
    offset += KDF_PARAMS_LENGTH;

    // Nonce
    let nonce = &data[offset..offset + NONCE_LENGTH];
    offset += NONCE_LENGTH;

    // Ciphertext
    let ciphertext = &data[offset..];

    let key = kdf::derive_key(password, salt, &kdf_params)?;
    let plaintext =
        aead::decrypt(&key, nonce, ciphertext).map_err(|_| VaulturaError::WrongPassword)?;

    let payload: VaultPayload = bincode::deserialize(&plaintext)?;
    Ok((payload, kdf_params))
}

/// Read vault file without decrypting — just extract the KDF params and salt for UI feedback.
pub fn read_vault_header(path: &Path) -> Result<(Vec<u8>, KdfParams)> {
    let data = fs::read(path)?;

    if data.len() < MIN_FILE_SIZE {
        return Err(VaulturaError::InvalidVaultFile {
            reason: "File too small".to_string(),
        });
    }

    if &data[0..4] != MAGIC {
        return Err(VaulturaError::InvalidVaultFile {
            reason: "Invalid magic bytes".to_string(),
        });
    }

    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(VaulturaError::InvalidVaultFile {
            reason: format!("Unsupported version: {version}"),
        });
    }

    let salt = data[8..8 + SALT_LENGTH].to_vec();
    let kdf_params = read_kdf_params(&data[8 + SALT_LENGTH..8 + SALT_LENGTH + KDF_PARAMS_LENGTH]);
    Ok((salt, kdf_params))
}

fn write_kdf_params(data: &mut Vec<u8>, params: &KdfParams) {
    data.extend_from_slice(&params.memory_cost_kib.to_le_bytes());
    data.extend_from_slice(&params.time_cost.to_le_bytes());
    data.extend_from_slice(&params.parallelism.to_le_bytes());
}

fn read_kdf_params(data: &[u8]) -> KdfParams {
    KdfParams {
        memory_cost_kib: u32::from_le_bytes(data[0..4].try_into().unwrap()),
        time_cost: u32::from_le_bytes(data[4..8].try_into().unwrap()),
        parallelism: u32::from_le_bytes(data[8..12].try_into().unwrap()),
    }
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let temp_path = parent.join(format!(".vaultura_tmp_{}", std::process::id()));

    let mut file = fs::File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;
    drop(file);

    fs::rename(&temp_path, path)?;
    Ok(())
}

/// Export vault: re-encrypts current payload with a different password.
pub fn export_vault(
    path: &Path,
    password: &str,
    kdf_params: &KdfParams,
    payload: &VaultPayload,
) -> Result<()> {
    write_vault(path, password, kdf_params, payload)
}

/// Import vault: reads a vault file with the given password.
pub fn import_vault(path: &Path, password: &str) -> Result<VaultPayload> {
    let (payload, _) = read_vault(path, password)?;
    Ok(payload)
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

    #[test]
    fn test_create_and_read_vault() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let params = test_params();

        create_vault(&path, "master_password", &params).unwrap();
        let (payload, read_params) = read_vault(&path, "master_password").unwrap();

        assert!(payload.groups.is_empty());
        assert!(payload.items.is_empty());
        assert_eq!(payload.meta.version, 1);
        assert_eq!(read_params, params);
    }

    #[test]
    fn test_write_and_read_with_data() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let params = test_params();

        let mut payload = VaultPayload::default();
        let group = crate::core::models::Group::new("Test".to_string(), None);
        let item = crate::core::models::Item::new("Login".to_string(), Some(group.id));
        payload.groups.push(group);
        payload.items.push(item);

        write_vault(&path, "password", &params, &payload).unwrap();
        let (read_payload, _) = read_vault(&path, "password").unwrap();
        assert_eq!(read_payload, payload);
    }

    #[test]
    fn test_wrong_password() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let params = test_params();

        create_vault(&path, "correct", &params).unwrap();
        let result = read_vault(&path, "wrong");
        assert!(matches!(result, Err(VaulturaError::WrongPassword)));
    }

    #[test]
    fn test_corrupted_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");

        fs::write(&path, b"garbage data that is not a vault").unwrap();
        let result = read_vault(&path, "password");
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");

        fs::write(&path, b"VLT").unwrap();
        let result = read_vault(&path, "password");
        assert!(matches!(
            result,
            Err(VaulturaError::InvalidVaultFile { .. })
        ));
    }

    #[test]
    fn test_read_vault_header() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.vault");
        let params = test_params();

        create_vault(&path, "password", &params).unwrap();
        let (salt, read_params) = read_vault_header(&path).unwrap();
        assert_eq!(salt.len(), SALT_LENGTH);
        assert_eq!(read_params, params);
    }

    #[test]
    fn test_export_import() {
        let dir = TempDir::new().unwrap();
        let original_path = dir.path().join("original.vault");
        let export_path = dir.path().join("export.vault");
        let params = test_params();

        let mut payload = VaultPayload::default();
        payload
            .groups
            .push(crate::core::models::Group::new("G".to_string(), None));
        write_vault(&original_path, "pass1", &params, &payload).unwrap();

        let (read_payload, _) = read_vault(&original_path, "pass1").unwrap();
        export_vault(&export_path, "pass2", &params, &read_payload).unwrap();

        let imported = import_vault(&export_path, "pass2").unwrap();
        assert_eq!(imported, payload);
    }
}
