use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use secrecy::SecretBox;

use crate::core::models::KdfParams;
use crate::error::{Result, VaulturaError};

const KEY_LENGTH: usize = 32;

pub fn generate_salt(len: usize) -> Vec<u8> {
    let mut salt = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8], params: &KdfParams) -> Result<SecretBox<Vec<u8>>> {
    let argon2_params = Params::new(
        params.memory_cost_kib,
        params.time_cost,
        params.parallelism,
        Some(KEY_LENGTH),
    )
    .map_err(|e| VaulturaError::Kdf(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut key = vec![0u8; KEY_LENGTH];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| VaulturaError::Kdf(e.to_string()))?;

    Ok(SecretBox::new(Box::new(key)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    fn test_params() -> KdfParams {
        KdfParams {
            memory_cost_kib: 1024,
            time_cost: 1,
            parallelism: 1,
        }
    }

    #[test]
    fn test_generate_salt_length() {
        let salt = generate_salt(32);
        assert_eq!(salt.len(), 32);
    }

    #[test]
    fn test_generate_salt_uniqueness() {
        let salt1 = generate_salt(32);
        let salt2 = generate_salt(32);
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let params = test_params();
        let salt = vec![0u8; 32];
        let key1 = derive_key("password", &salt, &params).unwrap();
        let key2 = derive_key("password", &salt, &params).unwrap();
        assert_eq!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let params = test_params();
        let salt = vec![0u8; 32];
        let key1 = derive_key("password1", &salt, &params).unwrap();
        let key2 = derive_key("password2", &salt, &params).unwrap();
        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn test_derive_key_different_salts() {
        let params = test_params();
        let salt1 = vec![0u8; 32];
        let salt2 = vec![1u8; 32];
        let key1 = derive_key("password", &salt1, &params).unwrap();
        let key2 = derive_key("password", &salt2, &params).unwrap();
        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn test_derive_key_length() {
        let params = test_params();
        let salt = generate_salt(32);
        let key = derive_key("password", &salt, &params).unwrap();
        assert_eq!(key.expose_secret().len(), KEY_LENGTH);
    }
}
