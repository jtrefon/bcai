//! Defines the `ChunkId`, a content-addressable identifier for a chunk.

use super::error::ChunkError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Unique identifier for a data chunk (SHA-256 hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ChunkId(pub String);

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_hash())
    }
}

impl From<&str> for ChunkId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl ChunkId {
    /// Create a new ChunkId from a slice of bytes.
    pub fn from_data(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        Self(format!("{:x}", hash))
    }

    /// Create a ChunkId from a hexadecimal string.
    pub fn from_hex(hex: &str) -> Result<Self, ChunkError> {
        if hex.len() != 64 {
            return Err(ChunkError::InvalidHash(
                "Hash must be 64 hex characters".to_string(),
            ));
        }

        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ChunkError::InvalidHash("Invalid hex characters".to_string()));
        }

        Ok(Self(hex.to_string()))
    }

    /// Get the full hash as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get a shortened version of the hash for display purposes.
    pub fn short_hash(&self) -> &str {
        &self.0.get(..8).unwrap_or_default()
    }
} 