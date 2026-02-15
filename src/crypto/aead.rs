use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretBox};

use crate::error::{Result, VaulturaError};

const NONCE_LENGTH: usize = 24;

pub fn encrypt(key: &SecretBox<Vec<u8>>, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.expose_secret())
        .map_err(|e| VaulturaError::Encryption(e.to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VaulturaError::Encryption(e.to_string()))?;

    Ok((nonce_bytes.to_vec(), ciphertext))
}

pub fn decrypt(key: &SecretBox<Vec<u8>>, nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.expose_secret())
        .map_err(|e| VaulturaError::Decryption(e.to_string()))?;

    let nonce = XNonce::from_slice(nonce);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| VaulturaError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SecretBox<Vec<u8>> {
        SecretBox::new(Box::new(vec![0x42u8; 32]))
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"Hello, Vaultura!";
        let (nonce, ciphertext) = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key = test_key();
        let wrong_key = SecretBox::new(Box::new(vec![0x99u8; 32]));
        let plaintext = b"secret data";
        let (nonce, ciphertext) = encrypt(&key, plaintext).unwrap();
        let result = decrypt(&wrong_key, &nonce, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"secret data";
        let (nonce, mut ciphertext) = encrypt(&key, plaintext).unwrap();
        ciphertext[0] ^= 0xFF;
        let result = decrypt(&key, &nonce, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_length() {
        let key = test_key();
        let (nonce, _) = encrypt(&key, b"test").unwrap();
        assert_eq!(nonce.len(), NONCE_LENGTH);
    }

    #[test]
    fn test_different_nonces_per_encryption() {
        let key = test_key();
        let (nonce1, _) = encrypt(&key, b"test").unwrap();
        let (nonce2, _) = encrypt(&key, b"test").unwrap();
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = test_key();
        let (nonce, ciphertext) = encrypt(&key, b"").unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_large_plaintext() {
        let key = test_key();
        let plaintext = vec![0xABu8; 1_000_000];
        let (nonce, ciphertext) = encrypt(&key, &plaintext).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
