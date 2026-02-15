/// Magic bytes identifying a Vaultura vault file: "VLTR"
pub const MAGIC: &[u8; 4] = b"VLTR";

/// Current vault file format version.
pub const VERSION: u32 = 1;

/// Length of the salt in bytes.
pub const SALT_LENGTH: usize = 32;

/// Length of the XChaCha20-Poly1305 nonce in bytes.
pub const NONCE_LENGTH: usize = 24;

/// KDF params are serialized as 3 x u32 = 12 bytes.
pub const KDF_PARAMS_LENGTH: usize = 12;

/// Minimum file size: magic(4) + version(4) + salt(32) + kdf_params(12) + nonce(24) + at least 1 byte ciphertext.
pub const MIN_FILE_SIZE: usize = 4 + 4 + SALT_LENGTH + KDF_PARAMS_LENGTH + NONCE_LENGTH + 1;
