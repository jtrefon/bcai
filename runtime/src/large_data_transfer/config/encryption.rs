use serde::{Deserialize, Serialize};

/// Supported encryption algorithms for data-at-rest / data-in-flight.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    None,
    ChaCha20Poly1305,
    Aes256Gcm,
}

/// Encryption configuration controlling optional encryption of chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable or disable encryption.
    pub enabled: bool,
    /// Encryption algorithm to use when enabled.
    pub algorithm: EncryptionAlgorithm,
    /// If `true`, individual chunks are encrypted; otherwise the full stream.
    pub chunk_encryption: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
            chunk_encryption: false,
        }
    }
} 