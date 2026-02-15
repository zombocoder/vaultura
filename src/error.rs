use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaulturaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Invalid vault file: {reason}")]
    InvalidVaultFile { reason: String },

    #[error("Wrong master password")]
    WrongPassword,

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("KDF error: {0}")]
    Kdf(String),

    #[error("Item not found: {0}")]
    ItemNotFound(uuid::Uuid),

    #[error("Group not found: {0}")]
    GroupNotFound(uuid::Uuid),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, VaulturaError>;
